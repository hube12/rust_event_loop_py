use crate::{
    audit_ffi::{
        create_heap_pointer,
        ffi_bytes_array,
        ffi_light_event::FFILightEvent,
        FFIArray,
        FFIError,
        FFIRuntime,
        FFISafe,
    },
    subscriber::SubscriberCallback,
    SEvent,
    SEventType,
    SubscribeHandle,
};
use anyhow::anyhow;
use audit_assembler::ResultMsg;
use audit_constants::AuditEventType;
use audit_errors::ParsingError;
use audit_merging::{
    Actor,
    Address,
    Entity,
    File,
    Network,
    NormalizedFields,
    Object,
    Process,
    Summary,
    User,
};
use std::collections::HashMap;
use std::panic::catch_unwind;
use crate::audit_ffi::FFINull;

pub struct FFISubscriber(
    pub(in crate::audit_ffi) SubscribeHandle<SEventType, Box<dyn SubscriberCallback<SEvent>>>,
);

impl FFISafe for FFISubscriber {
}

#[allow(dead_code)]
impl FFISubscriber {
    #[no_mangle]
    pub extern "C" fn subscribe(
        handle: *mut Self,
        event_type: *const SEventType,
        callback: extern "C" fn(*mut FFISEvent),
        runtime: *mut FFIRuntime,
    ) -> FFIError<FFINull> {
        let rt = match FFIRuntime::safe(runtime) {
            Ok(rt) => rt,
            Err(error) => return error,
        };
        let handle = match FFISubscriber::safe(handle) {
            Ok(x) => x,
            Err(error) => return error,
        };
        let event_type=match unsafe { event_type.as_ref() } {
            None => return FFIError::from(anyhow::Error::msg("Null pointer provided as an argument for event_type")),
            Some(event_type) => event_type.clone()
        };
        if let Err(e) = rt.runtime().handle().block_on(handle.0.subscribe(
            event_type,
            Box::new(move |e| {
                let event = create_heap_pointer(FFISEvent::from(e));
                callback(event);
            }),
        )) {
            let typ = e.0 .0;
            return FFIError::from(anyhow!(
                "Could not create a callback for this type {:?}",
                typ
            ));
        }
        FFIError::from_value(FFINull)
    }
}

#[repr(C)]
pub enum FFISEvent {
    ErrorChannelAssembler(FFIArray<std::os::raw::c_uchar>),
    ErrorChannelClient(FFIArray<std::os::raw::c_uchar>),
    ErrorProcessus(FFIArray<std::os::raw::c_uchar>),
    Event(FFILightEvent),
    ResultMsg(FFIResultMsg),
    Kill,
    Unknown,
}

impl From<SEvent> for FFISEvent {
    fn from(event: SEvent) -> Self {
        match event {
            SEvent::ErrorChannelAssembler(s) => {
                Self::ErrorChannelAssembler(ffi_bytes_array(s.as_bytes()))
            },
            SEvent::ErrorChannelClient(s) => {
                Self::ErrorChannelClient(ffi_bytes_array(s.as_bytes()))
            },
            SEvent::ErrorProcessus(s) => Self::ErrorProcessus(ffi_bytes_array(s.as_bytes())),
            SEvent::Event(e) => Self::Event((*e).into()),
            SEvent::ResultMsg(msg) => match msg {
                ResultMsg::DropCount(c) => Self::ResultMsg(FFIResultMsg::DropCount(c)),
                ResultMsg::SeqCount(c) => Self::ResultMsg(FFIResultMsg::SeqCount(c)),
                ResultMsg::Closed => Self::ResultMsg(FFIResultMsg::ChanClosed),
                _ => Self::Unknown,
            },
            SEvent::ResultCmd(_) => Self::Unknown,
            SEvent::Kill => Self::Kill,
        }
    }
}

#[repr(C)]
pub enum FFIResultMsg {
    DropCount(u32),
    SeqCount(usize),
    ChanClosed,
}

#[no_mangle]
pub extern "C" fn test_ev() -> FFILightEvent {
    let ev = audit_merging::Event {
        timestamp:         0,
        sequence:          0,
        category:          AuditEventType::EventTypeUnknown,
        audit_type:        0,
        result:            "123".to_string(),
        session:           Some(123321),
        tags:              vec!["123123312".to_string(), "12312".to_string()],
        summary:           Some(Summary {
            actor:  Some(Actor {
                primary:   "wqeqw".to_string(),
                secondary: "qeqew".to_string(),
            }),
            action: "test".to_string(),
            object: Some(Object {
                typ:       "wqeqew".to_string(),
                primary:   "weqew".to_string(),
                secondary: "qwqew".to_string(),
            }),
            how:    "test".to_string(),
        }),
        user:              Some(User {
            uid: None,
            username: "".to_string(),
            ids:     HashMap::from([("qewwqe".to_string(), "qweewq".to_string())]),
            names:   HashMap::from([
                ("12312".to_string(), "eqwe".to_string()),
                ("vv".to_string(), "eqwe".to_string()),
                ("eqe".to_string(), "13223".to_string()),
            ]),
            selinux: HashMap::from([("123123".to_string(), "qewew".to_string())]),
        }),
        process:           Some(Process {
            pid:   "eq".to_string(),
            ppid:  "qwe".to_string(),
            title: "qew".to_string(),
            name:  "1".to_string(),
            exe:   "312".to_string(),
            cwd:   "312".to_string(),
            hash_md5: None,
            hash_sha1: None,
            hash_sha256: None,
            args:  vec!["12321332".to_string(), "qwqwew".to_string()],
            exit_code: "1".to_string(),
            cgroups: vec![],
            container_id: "".to_string(),
            syscall: "qweqwe".to_string(),
            registers: vec!["wqqwe".to_string(),"qweqwqew".to_string()]
        }),
        file:              Some(File {
            path:    "123".to_string(),
            device:  "132ew1q".to_string(),
            inode:   "123".to_string(),
            mode:    "qw".to_string(),
            uid:     "a".to_string(),
            gid:     "12e12e".to_string(),
            owner:   "qweqew".to_string(),
            group:   "123132".to_string(),
            selinux: HashMap::from([("123123".to_string(), "qewew".to_string())]),
        }),
        src:               Some(Address {
            hostname: "1232".to_string(),
            ip:       "13232132".to_string(),
            port:     "12332123".to_string(),
            path:     "132132".to_string(),
        }),
        dst:               Some(Address {
            hostname: "1232".to_string(),
            ip:       "13232132".to_string(),
            port:     "12332123".to_string(),
            path:     "132132".to_string(),
        }),
        net:               Some(Network { direction: 0 }),
        data:              Some(HashMap::from([("123123".to_string(), "qewew".to_string())])),
        paths:             Some(vec![
            HashMap::from([("123123".to_string(), "qewew".to_string())]),
            HashMap::from([("123123".to_string(), "qewew".to_string())]),
        ]),
        normalized_fields: NormalizedFields {
            category:  Some(vec!["qewqew".to_string(), "12323".to_string()]),
            typ:       Some(vec!["qewqew".to_string(), "12323".to_string()]),
            outcome:   "32123".to_string(),
            user:      Entity {
                id:   Some("13312".to_string()),
                name: Some("21312332".to_string()),
            },
            effective: Entity {
                id:   Some("13312".to_string()),
                name: Some("21312332".to_string()),
            },
            target:    Entity {
                id:   Some("13312".to_string()),
                name: Some("21312332".to_string()),
            },
            changes:   Entity {
                id:   Some("13312".to_string()),
                name: Some("21312332".to_string()),
            },
            group:     Entity {
                id:   Some("13312".to_string()),
                name: Some("21312332".to_string()),
            },
        },
        hook_infos: vec![],
        warnings:          vec![
            ParsingError::ConfigMappingError("ewqwe".to_string()),
            ParsingError::InvalidAuditHeader,
        ],
    };
    ev.into()
}
