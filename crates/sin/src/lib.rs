use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Fields, FieldsNamed, GenericParam, Generics,
};

use proc_macro2::Span;
use quote::ToTokens;
use syn::parse::ParseStream;
use syn::Result;

#[derive(Debug)]
struct DbFields {
    fields: Vec<Rc<NoSqlField>>,
    partition_keys: Vec<FieldRef>,
    clustering_keys: Option<Vec<FieldRef>>,
}
#[derive(Debug)]
struct NoSqlField {
    // TODO : make this string wrapped in quotes when creating a query
    ident: syn::Ident,
    ty: syn::Type,
    //span : proc_macro2::Span,
}

#[derive(Debug)]
struct FieldRef {
    name: String,
    index: Option<Rc<NoSqlField>>,
}

impl From<String> for FieldRef {
    fn from(value: String) -> Self {
        Self {
            name: value,
            index: None,
        }
    }
}

use std::rc::Rc;

fn get_field_with_types<'a>(
    data: &'a Data,
    mut partition_keys: Vec<FieldRef>,
    mut clustering_keys: Option<Vec<FieldRef>>,
) -> Option<DbFields> {
    match *data {
        Data::Struct(ref data) => {
            if let Fields::Named(ref fields) = data.fields {
                let mut db_fields = Vec::with_capacity(fields.named.len());
                let _: Vec<()> = fields
                    .named
                    .iter()
                    .map(|f| {
                        f.ident.as_ref().map(|ident| {
                            let entry = Rc::new(NoSqlField {
                                ident: ident.clone(),
                                ty: f.ty.clone(),
                            });
                            db_fields.push(entry.clone());

                            let index = db_fields.len() - 1;

                            partition_keys
                                .iter_mut()
                                .find(|item| db_fields[index].ident == item.name)
                                .map(|found| found.index = Some(entry.clone()));

                            clustering_keys.as_deref_mut().map(|cluster_keys| {
                                cluster_keys
                                    .iter_mut()
                                    .find(|item| db_fields[index].ident == item.name)
                                    .map(|found| found.index = Some(entry.clone()))
                            });
                        });
                    })
                    .collect();
                return Some(DbFields {
                    fields: db_fields,
                    partition_keys,
                    clustering_keys,
                });
            }
            None
        }
        _ => None,
    }
}

#[proc_macro_derive(ToCqlData)]
pub fn derive_to_cql(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    match generate_derive_body(&input.data) {
        Ok(derive_body) => {
            let expanded = quote! {
                impl ToCqlData for #name{
                    fn to_cql(self) -> CqlType{
                        #derive_body
                    }
                }
            };
            proc_macro::TokenStream::from(expanded)
        }
        Err(error) => proc_macro::TokenStream::from(error),
    }
}

fn generate_derive_body(data: &Data) -> std::result::Result<TokenStream, TokenStream> {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let capacity = fields.named.len();
                let field_itr = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! {
                        f.span() =>
                            let value = ToCqlData::to_cql(self.#name);
                            res.insert(stringify!(#name).to_string(), value);
                    }
                });
                Ok(quote! {
                    let mut res : HashMap<String, CqlType> = HashMap::with_capacity(#capacity);
                    #(#field_itr)*
                    CqlType::Row(res)
                })
            }
            _ => Err(
                syn::Error::new(Span::call_site(), "unnamed structs not supported")
                    .to_compile_error(),
            ),
        },
        _ => Err(syn::Error::new(Span::call_site(), "only structs supported").to_compile_error()),
    }
}

fn get_fields<'a>(data: &'a Data) -> Option<&'a FieldsNamed> {
    match *data {
        Data::Struct(ref data) => {
            if let Fields::Named(ref fields) = data.fields {
                return Some(fields);
            }
            None
        }
        _ => None,
    }
}

#[proc_macro_derive(FromCqlData)]
pub fn derive_from_cql(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let name = input.ident;

    let fields: &FieldsNamed = match get_fields(&input.data) {
        Some(a) => a,
        None => {
            return proc_macro::TokenStream::from(
                syn::Error::new(Span::call_site(), "expected struct with named fields")
                    .to_compile_error(),
            )
        }
    };

    let try_from = try_from_struct(fields);
    let from_cql = from_cql_body();
    let expanded = quote! {

        impl TryFrom<&HashMap<String, CqlType>> for #name{
            type Error = ();
            fn try_from(map: &HashMap<String, CqlType>) -> Result<Self, Self::Error>{
                Ok(Self{
                    #try_from
                })
            }
        }

        impl FromCqlData for #name{
            type Error = String;

            fn from_cql(result : &CqlType) -> Result<Self, Self::Error>{
                #from_cql
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn try_from_struct(fields: &FieldsNamed) -> TokenStream {
    let expanded = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! {
            f.span() =>
                #name : {
                    let value = map.get(stringify!(#name)).ok_or(())?;
                    FromCqlData::from_cql(value)?
                },
        }
    });
    quote! {
        #(#expanded)*
    }
}

fn from_cql_body() -> TokenStream {
    quote! {
        match result {
            CqlType::Row(r) => {
                r.try_into().map_err(|_e| "type mismatch".to_string())
            },
            _ => Err("only expecting row variant".to_string())
        }
    }
}

#[proc_macro_derive(Gen)]
pub fn derive_gen(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = gen_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let bind_body = generate_body(&input.data);

    //panic!("{}", bind_body.to_string());
    let expanded = quote! {
        impl #impl_generics Gen for #name #ty_generics #where_clause{
            fn bind_insert_statement(&self, s : &mut Statement){
                #bind_body
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn generate_body(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let field_itr = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! {f.span() =>
                        let value = BindType::bind_the_type(&self.#name);
                        s.bind_by_name(stringify!(#name), value);
                    }
                });
                quote! {
                    #(#field_itr)*
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn gen_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(BindType));
        }
    }
    generics
}

#[derive(Default, Debug)]
struct Args {
    _primary_key: Option<Vec<FieldRef>>,
    _clustering_keys: Option<Vec<FieldRef>>,
    table_name: Option<String>,
    keyspace: Option<String>,
}

// impl TryFrom<Vec<ExprAssign>> for Args{
//     type Error = SinInputError;

//     fn try_from(value: Vec<ExprAssign>) -> std::result::Result<Self, Self::Error> {
//         let map = value.into_iter()
//             .map(|exp | {
//                 let left = get_left_path(exp.left)?;
//                 if left ==
//                 let right = get_right_info(exp);
//                 (left, right)
//             })
//             .collect::<HashMap<Ident, >>()
//         Ok(Self::default())
//     }
// }

// fn get_left_path(exp : Box<Expr>) -> std::result::Result<Ident, SinInputError>{
//     match *exp{
//         Path(path) => {
//             path.path.get_ident().map(|i| i.clone()).ok_or(())
//         },
//         _ => Err(()),
//     }
// }

// fn get_array_exp(exp : Box<Expr>) -> std::result::Result<(),SinInputError>{
//     match *exp{
//         Expr::Array(arr) =>,
//         _ => Err(())
//     }
// }
// }

use syn::parse::Parse;
/// #[read_functions(Table{pkey = (), skey = [(), ()], table_name = name, keyspace = name})]
impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut _primary_key = None;
        let mut _clustering_keys = None;
        let mut table_name = None;
        let mut keyspace = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            let _eq = input.parse::<syn::Token![=]>()?;

            match key.to_string().as_str() {
                "table" => {
                    let value: syn::Expr = input.parse()?;
                    table_name = Some(value.to_token_stream().to_string());
                }
                "partition_key" => {
                    let value: Vec<FieldRef> = input
                        .parse::<syn::ExprArray>()?
                        .elems
                        .into_iter()
                        .map(|e| e.to_token_stream().to_string().into())
                        .collect();

                    _primary_key = Some(value);
                }
                "clustering_key" => {
                    let value: Vec<FieldRef> = input
                        .parse::<syn::ExprArray>()?
                        .elems
                        .into_iter()
                        .map(|e| e.to_token_stream().to_string().into())
                        .collect();

                    _clustering_keys = Some(value);
                }
                "keyspace" => {
                    let value: syn::Expr = input.parse()?;
                    keyspace = Some(value.to_token_stream().to_string());
                }
                _ => {}
            }

            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(Self {
            _primary_key,
            _clustering_keys,
            table_name,
            keyspace,
        })
    }
}

#[proc_macro_attribute]
pub fn nosql(
    attrs: proc_macro::TokenStream,
    minput: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args: Args = parse_macro_input!(attrs);
    let input: DeriveInput = parse_macro_input!(minput);
    let table = args.table_name.expect("table name expected");
    let keyspace = args.keyspace.expect("keyspace expected");
    let name = input.ident.clone();

    let pre_req = quote! {
        #[derive(sin::ToCqlData, sin::FromCqlData)]
    };

    let query_traits = quote! {
        impl Selectable for #name{}
        impl Insertable for #name{}
    };

    let partition_keys = match args._primary_key {
        Some(k) => k,
        None => {
            return proc_macro::TokenStream::from(
                syn::Error::new(Span::call_site(), "primary keys not found").to_compile_error(),
            )
        }
    };
    let clustering_keys = args._clustering_keys;

    let fields = match get_field_with_types(&input.data, partition_keys, clustering_keys) {
        Some(a) => a,
        None => {
            return proc_macro::TokenStream::from(
                syn::Error::new(Span::call_site(), "expected struct with named fields")
                    .to_compile_error(),
            )
        }
    };

    let insert_statement = generate_insert(&table, &keyspace, fields.fields);

    let nosql = quote! {
        impl NoSql for #name {
            fn table_name() -> &'static str{
                #table
            }

            fn keyspace() -> &'static str{
                #keyspace
            }

            fn insert_statement() -> &'static str{
                #insert_statement
            }
        }

    };

    let filters = generate_filters(table, fields.partition_keys, fields.clustering_keys);

    let gen_filters = {
        quote! {
            impl #name{
               #filters
            }
        }
    };

    proc_macro::TokenStream::from(quote! {
        #pre_req
        #input
        #nosql
        #query_traits
        #gen_filters
    })
}

fn generate_insert(table: &str, keyspace: &str, fields: Vec<Rc<NoSqlField>>) -> String {
    let col_len = fields.len();
    let col: String = fields
        .into_iter()
        .map(|f: Rc<NoSqlField>| f.ident.to_string())
        .collect::<Vec<String>>()
        .join(",");
    let binds = std::iter::repeat("?")
        .take(col_len)
        .collect::<Vec<&str>>()
        .join(",");
    format!(
        "INSERT INTO {}.{} ({}) VALUES({})",
        keyspace, table, col, binds
    )
}

struct FilterByBuilder {
    table: String,
    data_map: Vec<FieldRef>,
    query_string: String,
    fn_prefix: String,
}

impl FilterByBuilder {
    //TODO: add approx for string size as well
    fn new(field_size: usize, table: String) -> Self {
        Self {
            table,
            data_map: Vec::with_capacity(field_size),
            query_string: String::new(),
            fn_prefix: String::from("filter_by"),
        }
    }

    fn add(&mut self, field: FieldRef) {
        self.fn_prefix.extend(["_", field.name.as_str()]);

        self.data_map.push(field);
    }
}

impl ToTokens for FilterByBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fn_sig = self.data_map.iter().map(|field| {
            let ident = &field.index.as_deref().unwrap().ident;
            let ty = &field.index.as_deref().unwrap().ty;

            quote! {
                #ident : #ty
            }
        });

        let fn_body = self.data_map.iter().map(|field| {
            let ident = &field.index.as_deref().unwrap().ident.to_string();
            let typ = &field.index.as_deref().unwrap().ty;

            quote! {
                (#ident.to_string(), #typ.to_cql())
            }
        });

        let body = generate_select_builder(&self.table, &self.data_map, fn_body);

        let fn_name = syn::Ident::new(&self.fn_prefix.as_str(), Span::call_site().into());

        let res = quote! {
            fn #fn_name (#(#fn_sig),*) -> SelectClause<Self>{
                #body
            }
        };

        tokens.extend(res);
    }
}

fn generate_select_builder(
    table: &str,
    refs: &[FieldRef],
    iter: impl Iterator<Item = TokenStream>,
) -> TokenStream {
    if refs.len() == 1 {
        let field_name = &refs.first().unwrap().name;

        return quote! {
            let filter = HashMap::from([#(#iter),*]);
            SelectBuilder::<Self>::new(filter).wh().eq(String::from(#field_name))
        };
    }

    let fields = refs.iter().map(|field| {
        let field_name = &field.name;

        quote! {
            .and().eq(String::from(#field_name))
        }
    });

    quote! {
        let filter = HashMap::from([#(#iter),*]);
        SelectBuilder::<Self>::new(#table.to_string(), filter).wh()#(#fields)*
    }
}

fn generate_filters(
    table: String,
    partition_keys: Vec<FieldRef>,
    clustering_keys: Option<Vec<FieldRef>>,
) -> TokenStream {
    let field_size = partition_keys.len()
        + clustering_keys
            .as_ref()
            .map(|v| v.len())
            .unwrap_or_default();
    let mut token_stream = TokenStream::new();
    let mut filter_builder = FilterByBuilder::new(field_size, table);

    for i in partition_keys.into_iter() {
        filter_builder.add(i);
    }
    filter_builder.to_tokens(&mut token_stream);

    if let Some(cluster_keys) = clustering_keys {
        for i in cluster_keys.into_iter() {
            filter_builder.add(i);
            filter_builder.to_tokens(&mut token_stream);
        }
    }

    token_stream
}
