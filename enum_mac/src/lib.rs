use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(OldEnumKeys)]
pub fn enum_keys_fn_old(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let empty: proc_macro::TokenStream = "".parse().unwrap();

	let mut token_iter = tokens.into_iter().peekable();

	let is_pub = token_iter.peek().unwrap().to_string() == "pub";
	if is_pub {
		token_iter.next();
	}

	let first = token_iter.next().unwrap();
	let second = token_iter.next().unwrap();
	// println!("First: {:?}", first);
	// println!("Second: {:?}", second);

	match first {
		proc_macro::TokenTree::Ident(ident) => {
			let i_name = ident.to_string();
			if i_name != "enum" {
				return empty;
			}
		}
		_ => return empty,
	}

	let name = match second {
		proc_macro::TokenTree::Ident(ident) => ident.to_string(),
		_ => return empty,
	};

	let key_group = match token_iter.next().unwrap() {
		proc_macro::TokenTree::Group(group) => group,
		_ => return empty,
	};

	let keys = key_group
		.stream()
		.into_iter()
		.filter(|f| match f {
			proc_macro::TokenTree::Ident(_) => true,
			_ => false,
		})
		.map(|f| match f {
			proc_macro::TokenTree::Ident(ident) => ident.to_string(),
			_ => panic!("Shouldn't happen"),
		})
		.collect::<Vec<_>>();

	let mut source_code = String::new();
	let mut match_chain = String::new();

	source_code += "#[derive(Debug, Copy, Clone)]\n";
	source_code += "#[repr(u8)]\n";
	if is_pub {
		source_code += "pub ";
	}
	source_code += &("enum ".to_owned() + &name + "Key {\n");

	let mut i: i32 = 0;
	for key in &keys {
		source_code += &("    ".to_owned() + &key + ",\n");

		match_chain += &("        ".to_owned() + &i.to_string() + " => " + &name + "Key::" + &key + ",\n");

		i += 1;
	}

	source_code += "}\n";
	source_code += "\n fn u8_to_";
	source_code += &name;
	source_code += "Key(value: u8) -> ";
	source_code += &name;
	source_code += "Key {\n";
	source_code += "    match value {\n";
	source_code += &match_chain;
	source_code += "        _ => panic!(\"Unknown ";
	source_code += &name;
	source_code += "Key: {}\", value),\n";
	source_code += "    }\n";
	source_code += "}\n";
	source_code += "\n";

	source_code.parse().unwrap()
}

#[proc_macro_derive(EnumKeys)]
pub fn enum_keys_fn(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let ast = parse_macro_input!(tokens as DeriveInput);

	let enum_data = match ast.data {
		syn::Data::Enum(data) => data,
		_ => panic!("EnumKeys can only be derived for enums"),
	};

	let enum_name = ast.ident.to_string() + "Key";
	let enum_name = Ident::new(&enum_name, ast.ident.span());
	let mut enum_keys = Vec::<TokenStream>::new();
	let mut enum_keys_to_u8 = Vec::<TokenStream>::new();
	let mut u8_to_enum_keys = Vec::<TokenStream>::new();

	let mut i: u8 = 0;
	for variant in enum_data.variants {
		let ident = variant.ident.clone();
		enum_keys.push(ident.to_token_stream());
		enum_keys_to_u8.push(quote! { #enum_name::#ident => #i, });
		u8_to_enum_keys.push(quote! { #i => #enum_name::#ident, });
		i += 1;
	}

	quote! {
		 pub enum #enum_name {
			  #(#enum_keys,)*
		 }

		 impl From<u8> for #enum_name {
			  fn from(key: u8) -> Self {
					match key {
						 #(#u8_to_enum_keys)*
						 _ => panic!("Unknown key: {}", key),
					}
			  }
		 }
	}
	.into()
}
