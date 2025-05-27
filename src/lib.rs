extern crate proc_macro2;

use quote::quote;

use std::env;
use std::path::Path;
use std::{fs::OpenOptions, process::Command};

use std::io::prelude::*;

mod types;

mod compile;

use crate::compile::compile_cdp_json;

pub const CDP_COMMIT: &str = "4f13107aac59fe418043f9edfdaef3b7da579614";

pub fn init() {
    init_with_commit(CDP_COMMIT);
}

pub fn init_with_commit(commit: &str) {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_file = Path::new(&out_dir).join("protocol.rs");
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&out_file)
        .unwrap();

    file.sync_all().unwrap();

    if file.metadata().unwrap().len() == 0 {
        let (js_mods, js_events) = compile_cdp_json("js_protocol.json", commit);

        let (browser_mods, browser_events) = compile_cdp_json("browser_protocol.json", commit);

        writeln!(
            file,
            "// Auto-generated from ChromeDevTools/devtools-protocol at commit {commit}"
        )
        .unwrap();

        let modv = quote! {
            pub mod types {
                use serde::{Deserialize, Serialize};
                use core::fmt::Debug;
            
                pub type JsFloat = f64;
                pub type JsUInt = u32;
            
                pub type WindowId = JsUInt;
            
                pub type CallId = JsUInt;
            
                #[derive(Serialize, Debug)]
                pub struct MethodCall<T>
                where
                    T: Debug,
                {
                    #[serde(rename = "method")]
                    method_name: &'static str,
                    pub id: CallId,
                    params: T,
                }
            
                impl<T> MethodCall<T>
                where
                    T: Debug,
                {
                    pub const fn get_params(&self) -> &T {
                        &self.params
                    }
                }
            
                pub trait Method: Debug {
                    const NAME: &'static str;
                
                    type ReturnObject: serde::de::DeserializeOwned + core::fmt::Debug;
                
                
                    fn to_method_call(self, call_id: CallId) -> MethodCall<Self>
                    where
                        Self: core::marker::Sized,
                    {
                            MethodCall {
                                id: call_id,
                                    params: self,
                                method_name: Self::NAME,
                            }
                    }
                }
            
                #[derive(Deserialize, Debug, Clone, PartialEq)]
                #[serde(tag = "method")]
                pub enum Event {
                    #(#browser_events)*
                    #(#js_events)*
                }
            }

            #(#js_mods)*
            #(#browser_mods)*
        };

        writeln!(file, "{modv}").unwrap();

        if env::var_os("DO_NOT_FORMAT").is_none() {
            let mut rustfmt = match env::var_os("RUSTFMT") {
                Some(rustfmt) => Command::new(rustfmt),
                None => Command::new("rustfmt"),
            };
            rustfmt.arg(&out_file).output().expect("rustfmt not found");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        crate::init();
    }
}
