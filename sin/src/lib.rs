use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Fields, FieldsNamed, GenericParam, Generics
};

use syn::Result;
use quote::ToTokens;
use syn::parse::ParseStream;
use proc_macro2::Span;

#[proc_macro_derive(ToCqlData)]
pub fn derive_to_cql(input : proc_macro::TokenStream) -> proc_macro::TokenStream{
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    match generate_derive_body(&input.data){
        Ok(derive_body) => {
            let expanded = quote! {
                impl ToCqlData for #name{
                    fn to_cql(self) -> CqlType{
                        #derive_body
                    }
                }
            };
            proc_macro::TokenStream::from(expanded)
        },
        Err(error) => {
            proc_macro::TokenStream::from(error)
        }
    }
}


fn generate_derive_body(data : &Data) -> std::result::Result<TokenStream, TokenStream>{
    match *data{
        Data::Struct(ref data) => {
            match data.fields{
                Fields::Named(ref fields) =>{
                    let capacity = fields.named.len();
                    let field_itr =
                        fields
                            .named
                            .iter()
                            .map(|f| {
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
                _ => Err(syn::Error::new(Span::call_site(), "unnamed structs not supported").to_compile_error()) 
            }
        }
        _ => Err(syn::Error::new(Span::call_site(), "only structs supported").to_compile_error()) 
    }
}

fn get_fields<'a>(data: &'a Data) -> Option<&'a FieldsNamed>{
    match *data{
        Data::Struct(ref data) => {
            if let Fields::Named(ref fields) = data.fields{
                return Some(fields)
            }
            None
        },
        _ => None
    }
}

#[proc_macro_derive(FromCqlData)]
pub fn derive_from_cql(input : proc_macro::TokenStream) -> proc_macro::TokenStream{
    let input: DeriveInput = parse_macro_input!(input);
    let name = input.ident;
    
    let fields :&FieldsNamed = match get_fields(&input.data){
        Some(a) => a,
        None => {
            return
                proc_macro::TokenStream::from(
                    syn::Error::new(Span::call_site(),
                        "expected struct with named fields")
                    .to_compile_error()
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

fn try_from_struct(fields : &FieldsNamed) -> TokenStream{
    let expanded = 
        fields.named
            .iter()
            .map(|f| {
                let name = &f.ident;
                quote_spanned! {
                    f.span() =>
                        #name : {
                            let value = map.get(stringify!(#name)).ok_or(())?;
                            FromCqlData::from_cql(value)?
                        },
                }
            });
    quote!{
        #(#expanded)*
    }
}

fn from_cql_body() -> TokenStream{
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
pub fn derive_gen(input : proc_macro::TokenStream) -> proc_macro::TokenStream{
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

fn generate_body(data : &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
             match data.fields {
                Fields::Named(ref fields) => {
                    let field_itr =
                        fields.named
                        .iter()
                        .map(|f| {
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
                _ => unimplemented!()
            }
        },
        _ => unimplemented!()
    }
}

fn gen_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param{
            type_param.bounds.push(parse_quote!(BindType));
        }
    }
    generics
}

#[derive(Default)]
struct Args{
    _primary_key : Option<Vec<String>>,
    _clustering_keys: Option<Vec<String>>,
    table_name: Option<String>,
    keyspace : Option<String>,
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
impl Parse for Args{
    fn parse(input: ParseStream) -> Result<Self>{
        let mut _primary_key = None;
        let mut _clustering_keys = None; 
        let mut table_name= None;
        let mut keyspace= None;

        while !input.is_empty(){
            let key: syn::Ident = input.parse()?;
            let _eq = input.parse::<syn::Token![=]>()?;

            match key.to_string().as_str(){
                "table" => {
                    let value : syn::Expr = input.parse()?;
                    table_name = Some(value.to_token_stream().to_string());
                },
                "partition_key" =>{
                    let value : Vec<String> =
                        input
                            .parse::<syn::ExprArray>()?
                            .elems
                            .into_iter()
                            .map(|e| e.to_token_stream().to_string())
                            .collect();
                    
                    _primary_key = Some(value);
                },
                "clustering_key" =>{
                    let value : Vec<String> =
                        input
                            .parse::<syn::ExprArray>()?
                            .elems
                            .into_iter()
                            .map(|e| e.to_token_stream().to_string())
                            .collect();
                    
                    _clustering_keys = Some(value);
                    
                },
                "keyspace" =>{
                    let value : syn::Expr = input.parse()?;
                    keyspace = Some(value.to_token_stream().to_string());
                }
                _ => {}
            }
            
            if !input.is_empty(){
                input.parse::<syn::Token![,]>()?;
            }
        }
        
        Ok(Self{
            _primary_key,
            _clustering_keys,
            table_name,
            keyspace
        })

    }
}

#[proc_macro_attribute]
pub fn nosql(attrs: proc_macro::TokenStream, minput : proc_macro::TokenStream) -> proc_macro::TokenStream{
    let args : Args = parse_macro_input!(attrs);
    let input: DeriveInput = parse_macro_input!(minput);
    let table = args.table_name.expect("table name expected");
    let keyspace = args.keyspace.expect("keyspace expected");
    let name = input.ident.clone();
    
    let pre_req = quote!{
        #[derive(sin::ToCqlData, sin::FromCqlData)]
    };

    let res = quote!{
        impl NoSql for #name {
            fn table_name() -> &'static str{
                #table
            }

            fn keyspace() -> &'static str{
                #keyspace
            }
        }

    };

    proc_macro::TokenStream::from(quote! {
        #pre_req
        #input
        #res
    })

}