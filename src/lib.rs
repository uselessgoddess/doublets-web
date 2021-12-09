#![feature(ptr_internals)]

mod utils;

use js_sys::Function;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type TLink = u64;

use doublets::doublets::mem::united;
use doublets::doublets::mem::splited;
use doublets::doublets::ILinks;
use doublets::mem::HeapMem;

#[wasm_bindgen]
pub struct Link {
    pub id: TLink,
    pub from_id: TLink,
    pub to_id: TLink,
}

#[wasm_bindgen]
impl Link {
    #[wasm_bindgen(constructor)]
    pub fn new(id: TLink, from_id: TLink, to_id: TLink) -> Self {
        Self { id, from_id, to_id }
    }
}

#[wasm_bindgen]
pub struct UnitedLinks {
    base: united::Links<TLink, HeapMem>,
}

#[wasm_bindgen]
impl UnitedLinks {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<UnitedLinks, JsValue> {
        Ok(Self {
            base: united::Links::<_, _>::new(
                HeapMem::new().map_err(|e| e.to_string())?
            ).map_err(|e| e.to_string())?
        })
    }

    pub fn create(&mut self) -> Result<TLink, JsValue> {
        Ok(
            self.base.create()
                .map_err(|e| e.to_string())?
        )
    }

    pub fn each_by(&mut self, closure: &js_sys::Function, query: Link) -> Result<TLink, JsValue> {
        Ok(self.base.each_by(|link| {
            let link = Link { // TODO: `impl From`
                id: link.index,
                from_id: link.source,
                to_id: link.target,
            };
            let this = JsValue::null();
            let result: JsValue = closure.call1(&this, &JsValue::from(link)).unwrap(); // TODO: [temp]
            let control = result.as_f64().expect("expected `number`"); // TODO: [super temp]
            control
        } as TLink, [query.id, query.from_id, query.to_id]))
    }

    pub fn update(&mut self, id: TLink, from_id: TLink, to_id: TLink) -> Result<TLink, JsValue> {
        Ok(
            self.base.update(id, from_id, to_id)
                .map_err(|e| e.to_string())?
        )
    }

    pub fn delete(&mut self, id: TLink) -> Result<TLink, JsValue> {
        Ok(
            self.base.delete(id)
                .map_err(|e| e.to_string())?
        )
    }
}
