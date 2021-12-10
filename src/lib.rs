#![feature(ptr_internals)]

mod utils;

use std::ops::RangeInclusive;
use js_sys::Function;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type TLink = u32;

use doublets::doublets::mem::united;
use doublets::doublets::mem::splited;
use doublets::doublets::ILinks;
use doublets::mem::HeapMem;

use doublets::doublets::Link as RealLink;
use doublets::doublets::data::LinksConstants as RealConstants;

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct LinkRange(pub TLink, pub TLink);

#[wasm_bindgen]
pub struct Link {
    pub id: TLink,
    pub from_id: TLink,
    pub to_id: TLink,
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct LinksConstants {
    pub index_part: TLink,
    pub source_part: TLink,
    pub target_part: TLink,
    #[wasm_bindgen(js_name = "_break")]
    pub r#break: TLink,
    #[wasm_bindgen(js_name = "_null")]
    pub null: TLink,
    #[wasm_bindgen(js_name = "_continue")]
    pub r#continue: TLink,
    pub skip: TLink,
    pub any: TLink,
    pub itself: TLink,
    pub internal_range: LinkRange,
    pub external_range: Option<LinkRange>,
}

pub mod const_utils {
    use super::*;

    pub fn from(real: RealConstants<TLink>) -> LinksConstants {
        LinksConstants {
            index_part: real.index_part,
            source_part: real.source_part,
            target_part: real.target_part,
            r#break: real.r#break,
            null: real.null,
            r#continue: real.r#continue,
            skip: real.skip,
            any: real.any,
            itself: real.itself,
            internal_range: LinkRange(*real.internal_range.start(), *real.internal_range.end()),
            external_range: (real.external_range.map(|e| LinkRange(*e.start(), *e.end()))),
        }
    }

    pub fn to(_self: LinksConstants) -> RealConstants<TLink> {
        RealConstants {
            index_part: _self.index_part,
            source_part: _self.source_part,
            target_part: _self.target_part,
            r#break: _self.r#break,
            null: _self.null,
            r#continue: _self.r#continue,
            skip: _self.skip,
            any: _self.any,
            itself: _self.itself,
            internal_range: RangeInclusive::new(_self.internal_range.0, _self.internal_range.1),
            external_range: (_self.external_range.map(|e| RangeInclusive::new(e.0, e.1))),
        }
    }
}

#[wasm_bindgen]
impl LinksConstants {
    // TODO: #[wasm_bindgen(constructor)]
    pub fn full_new(
        target_part: TLink,
        internal: LinkRange,
        external: Option<LinkRange>,
    ) -> Self {
        const_utils::from(RealConstants::full_new(
            target_part,
            RangeInclusive::new(internal.0, internal.1),
            external.map(|e| RangeInclusive::new(e.0, e.1)),
        ))
    }

    // TODO: #[wasm_bindgen(constructor)]
    pub fn via_external(target_part: TLink, external: bool) -> Self {
        const_utils::from(RealConstants::via_external(target_part, external))
    }

    // TODO: #[wasm_bindgen(constructor)]
    pub fn via_ranges(internal: LinkRange, external: Option<LinkRange>) -> Self {
        const_utils::from(RealConstants::full_new(
            2,
            RangeInclusive::new(internal.0, internal.1),
            external.map(|e| RangeInclusive::new(e.0, e.1)),
        ))
    }

    // TODO: #[wasm_bindgen(constructor)]
    pub fn via_only_external(external: bool) -> Self {
        const_utils::from(RealConstants::via_external(2, external))
    }

    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        const_utils::from(RealConstants::via_only_external(false))
    }

    pub fn clone(&self) -> Self {
        Clone::clone(self)
    }
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
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[deprecated]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}


#[wasm_bindgen]
impl UnitedLinks {
    #[wasm_bindgen(constructor)]
    // TODO: Make options constructor
    pub fn new(constants: Option<LinksConstants>) -> Result<UnitedLinks, JsValue> {
        Ok(Self {
            base: united::Links::<_, _>::with_constants(
                HeapMem::new().map_err(|e| e.to_string())?,
                constants.map_or(RealConstants::default(), |c| const_utils::to(c))
            ).map_err(|e| e.to_string())?
        })
    }

    pub fn create(&mut self) -> Result<TLink, JsValue> {
        Ok(
            self.base.create()
                .map_err(|e| e.to_string())?
        )
    }

    #[wasm_bindgen(getter)]
    pub fn constants(&mut self) -> LinksConstants {
        const_utils::from(self.base.constants.clone())
    }

    pub fn count(&mut self, query: Option<Link>) -> TLink {
        let any = self.base.constants.any;
        let query = query.unwrap_or(Link {
            id: any,
            from_id: any,
            to_id:any
        });
        self.base.count_by([query.id, query.from_id, query.to_id])
    }

    pub fn each(&mut self, closure: &js_sys::Function, query: Option<Link>) -> Result<TLink, JsValue> {
        let any = self.base.constants.any;
        let query = query.unwrap_or(Link {
            id: any,
            from_id: any,
            to_id:any
        });
        let mut error = None;
        let each = self.base.each_by(|link| {
            let link = Link { // TODO: `impl From`
                id: link.index,
                from_id: link.source,
                to_id: link.target,
            };
            let this = JsValue::null();
            let result: Result<JsValue, JsValue> = closure.call1(&this, &JsValue::from(link));
            match result {
                Err(err) => {
                    error = Some(err);
                    0
                }
                Ok(result) => {
                    if let Some(result) = result.as_f64() {
                        result as TLink
                    } else {
                        // TODO "... found `...`"
                        error = Some(JsValue::from_str("expected `number`"));
                        0
                    }
                }
            }
        } as TLink, [query.id, query.from_id, query.to_id]);

        if let Some(err) = error {
            Err(err)
        } else {
            Ok(each)
        }
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
