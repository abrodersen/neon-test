#[macro_use]
extern crate neon;
extern crate futures;

use std::mem;
use std::sync::mpsc;

use neon::vm::{Call, Lock, JsResult};
use neon::js::{JsValue, JsString, JsFunction, JsUndefined, JsObject, JsInteger, Object};
use neon::js::binary::{JsBuffer};
use neon::js::class::Class;
use neon::js::error::{JsError, Kind};
use neon::scope::{Scope, RootScope};
use neon::task::Task;

use futures::Future;

pub struct WritableBuffer {
    buf: Vec<u8>
}

declare_types! {
    pub class JsWritableBuffer for WritableBuffer {
        init(_) {
            Ok(WritableBuffer {
                buf: Vec::new(),
            })
        }

        method write(call) {
            let scope = call.scope;
            let mut chunk = call.arguments.require(scope, 0)?.check::<JsBuffer>()?;
            let callback = call.arguments.require(scope, 2)?.check::<JsFunction>()?;

            let buffer = chunk.grab(|inner| {
                let mut tmp = Vec::with_capacity(inner.len());
                tmp.extend_from_slice(inner.as_slice());
                tmp
            });
            
            call.arguments.this(scope).grab(move |inner| {
                inner.buf.extend_from_slice(&buffer);
            });

            (WriteTask {}).schedule(callback);

            Ok(JsUndefined::new().upcast())
        }

        method size(call) {
            let scope = call.scope;
            let size = call.arguments.this(scope).grab(|inner| inner.buf.len());
            let num = JsInteger::new(scope, size as i32);
            Ok(num.upcast())
        }
    }
}

struct WriteTask;

impl Task for WriteTask {
    type Output = ();
    type Error = &'static str;
    type JsEvent = JsUndefined;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    fn complete<'a, S: Scope<'a>>(self, _: &'a mut S, _: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        Ok(JsUndefined::new())
    }
}

struct FutureTask {
    recv: mpsc::Receiver<String>,
}

impl Task for FutureTask {
    type Output = String;
    type Error = &'static str;
    type JsEvent = JsString;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        self.recv.recv()
            .map_err(|_| "unable to receive data")
    }

    fn complete<'a, S: Scope<'a>>(self, scope: &'a mut S, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        let message = result
            .or_else(|e| JsError::throw(Kind::Error, e))?;

        JsString::new_or_throw(scope, &message)
    }
}

fn callback_hell(call: Call) -> JsResult<JsValue> {
    let scope = call.scope;
    let first_callback = call.arguments.require(scope, 0)?.check::<JsFunction>()?;
    let second_callback = call.arguments.require(scope, 1)?.check::<JsFunction>()?;

    let (sender, receiver) = mpsc::channel();

    let continuation = JsFunction::new::<RootScope, JsValue>(scope, Box::new(move |call| {
        let scope = call.scope;
        let message = call.arguments.require(scope, 0)?.check::<JsString>()?;
        sender.send(message.value())
            .or_else(|_| JsError::throw(Kind::Error, "unable to send data"))?;

        Ok(JsUndefined::new().upcast())
    }))?;

    (FutureTask { recv: receiver }).schedule(second_callback);
    
    first_callback.call(scope, first_callback, vec![continuation])?;

    Ok(JsUndefined::new().upcast())
}

    

register_module!(m, {

    let func = JsFunction::new(m.scope, Box::new(callback_hell))?;
    m.exports.set("cb", func)?;

    let constructor = JsWritableBuffer::class(m.scope)?.constructor(m.scope)?;
    m.exports.set("WritableBuffer", constructor)?;

    Ok(())
});
