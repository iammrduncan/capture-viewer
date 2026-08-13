#![cfg_attr(not(target_os = "macos"), allow(dead_code, unused_imports))]

#[cfg(not(target_os = "macos"))]
compile_error!("Capture Viewer is a native macOS application.");

use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr;

type Id = *mut c_void;
type Class = *mut c_void;
type Sel = *const c_void;
type ObjcBool = i8;

const NIL: Id = ptr::null_mut();
const NO: ObjcBool = 0;
const YES: ObjcBool = 1;
const AUTH_NOT_DETERMINED: isize = 0;
const AUTH_RESTRICTED: isize = 1;
const AUTH_DENIED: isize = 2;
const AUTH_AUTHORIZED: isize = 3;

#[repr(C)]
#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Size {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Rect {
    origin: Point,
    size: Size,
}

#[repr(C)]
struct ObjcSuper {
    receiver: Id,
    super_class: Class,
}

#[repr(C)]
struct BlockDescriptor {
    reserved: usize,
    size: usize,
}

#[repr(C)]
struct PermissionBlock {
    isa: *const c_void,
    flags: i32,
    reserved: i32,
    invoke: extern "C" fn(*mut PermissionBlock, ObjcBool),
    descriptor: *const BlockDescriptor,
}

static BLOCK_DESCRIPTOR: BlockDescriptor = BlockDescriptor {
    reserved: 0,
    size: std::mem::size_of::<PermissionBlock>(),
};

#[link(name = "objc")]
extern "C" {
    fn objc_getClass(name: *const c_char) -> Class;
    fn sel_registerName(name: *const c_char) -> Sel;
    fn objc_msgSend();
    fn objc_msgSendSuper();
    #[cfg(target_arch = "x86_64")]
    fn objc_msgSend_stret();
    fn objc_allocateClassPair(superclass: Class, name: *const c_char, extra_bytes: usize) -> Class;
    fn objc_registerClassPair(class: Class);
    fn class_addMethod(
        class: Class,
        selector: Sel,
        implementation: *const c_void,
        types: *const c_char,
    ) -> ObjcBool;
    fn objc_autoreleasePoolPush() -> *mut c_void;
    fn objc_autoreleasePoolPop(pool: *mut c_void);
    static _NSConcreteGlobalBlock: c_void;
}

#[link(name = "AppKit", kind = "framework")]
extern "C" {}

#[link(name = "QuartzCore", kind = "framework")]
extern "C" {}

#[link(name = "AVFoundation", kind = "framework")]
extern "C" {
    static AVMediaTypeVideo: Id;
    static AVMediaTypeAudio: Id;
    static AVLayerVideoGravityResizeAspect: Id;
    static AVCaptureDeviceWasConnectedNotification: Id;
    static AVCaptureDeviceWasDisconnectedNotification: Id;
}

unsafe fn selector(name: &'static [u8]) -> Sel {
    sel_registerName(name.as_ptr().cast())
}

unsafe fn class(name: &'static [u8]) -> Class {
    objc_getClass(name.as_ptr().cast())
}

macro_rules! msg {
    ($receiver:expr, $selector:literal => $return_type:ty) => {{
        let function: unsafe extern "C" fn(Id, Sel) -> $return_type =
            std::mem::transmute(objc_msgSend as *const ());
        function(
            $receiver as Id,
            selector(concat!($selector, "\0").as_bytes()),
        )
    }};
    ($receiver:expr, $selector:literal, $a:expr ; $a_type:ty => $return_type:ty) => {{
        let function: unsafe extern "C" fn(Id, Sel, $a_type) -> $return_type =
            std::mem::transmute(objc_msgSend as *const ());
        function(
            $receiver as Id,
            selector(concat!($selector, "\0").as_bytes()),
            $a,
        )
    }};
    ($receiver:expr, $selector:literal, $a:expr ; $a_type:ty, $b:expr ; $b_type:ty => $return_type:ty) => {{
        let function: unsafe extern "C" fn(Id, Sel, $a_type, $b_type) -> $return_type =
            std::mem::transmute(objc_msgSend as *const ());
        function(
            $receiver as Id,
            selector(concat!($selector, "\0").as_bytes()),
            $a,
            $b,
        )
    }};
    ($receiver:expr, $selector:literal, $a:expr ; $a_type:ty, $b:expr ; $b_type:ty, $c:expr ; $c_type:ty => $return_type:ty) => {{
        let function: unsafe extern "C" fn(Id, Sel, $a_type, $b_type, $c_type) -> $return_type =
            std::mem::transmute(objc_msgSend as *const ());
        function(
            $receiver as Id,
            selector(concat!($selector, "\0").as_bytes()),
            $a,
            $b,
            $c,
        )
    }};
    ($receiver:expr, $selector:literal, $a:expr ; $a_type:ty, $b:expr ; $b_type:ty, $c:expr ; $c_type:ty, $d:expr ; $d_type:ty => $return_type:ty) => {{
        let function: unsafe extern "C" fn(
            Id,
            Sel,
            $a_type,
            $b_type,
            $c_type,
            $d_type,
        ) -> $return_type = std::mem::transmute(objc_msgSend as *const ());
        function(
            $receiver as Id,
            selector(concat!($selector, "\0").as_bytes()),
            $a,
            $b,
            $c,
            $d,
        )
    }};
}

unsafe fn send_rect(receiver: Id, name: &'static [u8]) -> Rect {
    let sel = selector(name);
    #[cfg(target_arch = "x86_64")]
    {
        let mut value = std::mem::MaybeUninit::<Rect>::uninit();
        let function: unsafe extern "C" fn(*mut Rect, Id, Sel) =
            std::mem::transmute(objc_msgSend_stret as *const ());
        function(value.as_mut_ptr(), receiver, sel);
        value.assume_init()
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let function: unsafe extern "C" fn(Id, Sel) -> Rect =
            std::mem::transmute(objc_msgSend as *const ());
        function(receiver, sel)
    }
}

unsafe fn ns_string(value: &str) -> Id {
    let sanitized = value.replace('\0', "");
    let c_value = CString::new(sanitized).unwrap();
    msg!(
        class(b"NSString\0"),
        "stringWithUTF8String:",
        c_value.as_ptr(); *const c_char => Id
    )
}

unsafe fn rust_string(value: Id) -> String {
    if value.is_null() {
        return String::new();
    }
    let pointer = msg!(value, "UTF8String" => *const c_char);
    if pointer.is_null() {
        String::new()
    } else {
        CStr::from_ptr(pointer).to_string_lossy().into_owned()
    }
}

unsafe fn retain(value: Id) -> Id {
    if !value.is_null() {
        msg!(value, "retain" => Id)
    } else {
        NIL
    }
}

unsafe fn release(value: Id) {
    if !value.is_null() {
        msg!(value, "release" => ());
    }
}

struct AppState {
    controller: Id,
    window: Id,
    session: Id,
    preview_layer: Id,
    video_devices: Id,
    audio_devices: Id,
    video_input: Id,
    audio_input: Id,
    _audio_output: Id,
    source_menu: Id,
    audio_menu: Id,
    selected_video_uid: String,
    selected_audio_uid: Option<String>,
    session_started: bool,
}

static mut STATE: *mut AppState = ptr::null_mut();

unsafe fn state() -> &'static mut AppState {
    &mut *STATE
}

unsafe fn device_property(device: Id, property: &'static str) -> String {
    match property {
        "name" => rust_string(msg!(device, "localizedName" => Id)),
        "uid" => rust_string(msg!(device, "uniqueID" => Id)),
        "model" => rust_string(msg!(device, "modelID" => Id)),
        "manufacturer" => rust_string(msg!(device, "manufacturer" => Id)),
        _ => String::new(),
    }
}

unsafe fn array_count(array: Id) -> usize {
    if array.is_null() {
        0
    } else {
        msg!(array, "count" => usize)
    }
}

unsafe fn array_item(array: Id, index: usize) -> Id {
    msg!(array, "objectAtIndex:", index; usize => Id)
}

unsafe fn index_for_uid(array: Id, uid: &str) -> Option<usize> {
    for index in 0..array_count(array) {
        let device = array_item(array, index);
        if device_property(device, "uid") == uid {
            return Some(index);
        }
    }
    None
}

unsafe fn set_window_title(title: &str) {
    let window = state().window;
    msg!(window, "setTitle:", ns_string(title); Id => ());
}

unsafe fn new_menu_item(title: &str, action: Sel, key: &str) -> Id {
    let item = msg!(class(b"NSMenuItem\0"), "alloc" => Id);
    msg!(
        item,
        "initWithTitle:action:keyEquivalent:",
        ns_string(title); Id,
        action; Sel,
        ns_string(key); Id => Id
    )
}

unsafe fn add_menu_item(menu: Id, item: Id) {
    msg!(menu, "addItem:", item; Id => ());
    release(item);
}

unsafe fn placeholder_menu_item(menu: Id, title: &str) {
    let item = new_menu_item(title, ptr::null(), "");
    msg!(item, "setEnabled:", NO; ObjcBool => ());
    add_menu_item(menu, item);
}

unsafe fn refresh_source_checks() {
    let app_state = state();
    for index in 0..array_count(app_state.video_devices) {
        let item = msg!(app_state.source_menu, "itemAtIndex:", index; usize => Id);
        let uid = device_property(array_item(app_state.video_devices, index), "uid");
        let checked: isize = if uid == app_state.selected_video_uid {
            1
        } else {
            0
        };
        msg!(item, "setState:", checked; isize => ());
    }
}

unsafe fn refresh_audio_checks() {
    let app_state = state();
    let automatic = msg!(app_state.audio_menu, "itemAtIndex:", 0usize; usize => Id);
    let automatic_state: isize = if app_state.selected_audio_uid.is_none() {
        1
    } else {
        0
    };
    msg!(automatic, "setState:", automatic_state; isize => ());

    // Item zero is Automatic and item one is the separator.
    for index in 0..array_count(app_state.audio_devices) {
        let item = msg!(app_state.audio_menu, "itemAtIndex:", index + 2; usize => Id);
        let uid = device_property(array_item(app_state.audio_devices, index), "uid");
        let checked: isize = if app_state.selected_audio_uid.as_deref() == Some(uid.as_str()) {
            1
        } else {
            0
        };
        msg!(item, "setState:", checked; isize => ());
    }
}

fn normalized_device_name(name: &str) -> String {
    let mut normalized = name.to_ascii_lowercase();
    for word in ["video", "audio", "camera", "microphone", "capture", "input"] {
        normalized = normalized.replace(word, "");
    }
    normalized
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect()
}

unsafe fn has_audio(device: Id) -> bool {
    msg!(device, "hasMediaType:", AVMediaTypeAudio; Id => ObjcBool) == YES
}

unsafe fn automatic_audio_device(video_device: Id) -> Id {
    if has_audio(video_device) {
        return video_device;
    }

    if msg!(
        video_device,
        "respondsToSelector:",
        selector(b"linkedDevices\0"); Sel => ObjcBool
    ) == YES
    {
        let linked = msg!(video_device, "linkedDevices" => Id);
        for index in 0..array_count(linked) {
            let candidate = array_item(linked, index);
            if has_audio(candidate) {
                return candidate;
            }
        }
    }

    let app_state = state();
    let video_name = device_property(video_device, "name");
    let video_model = device_property(video_device, "model");
    let video_manufacturer = device_property(video_device, "manufacturer");

    for index in 0..array_count(app_state.audio_devices) {
        let candidate = array_item(app_state.audio_devices, index);
        if device_property(candidate, "name") == video_name {
            return candidate;
        }
    }

    if !video_model.is_empty() {
        for index in 0..array_count(app_state.audio_devices) {
            let candidate = array_item(app_state.audio_devices, index);
            if device_property(candidate, "model") == video_model
                && device_property(candidate, "manufacturer") == video_manufacturer
            {
                return candidate;
            }
        }
    }

    let normalized_video = normalized_device_name(&video_name);
    if normalized_video.len() >= 3 {
        for index in 0..array_count(app_state.audio_devices) {
            let candidate = array_item(app_state.audio_devices, index);
            let normalized_audio = normalized_device_name(&device_property(candidate, "name"));
            if normalized_audio == normalized_video {
                return candidate;
            }
        }
    }

    NIL
}

unsafe fn selected_audio_device(video_device: Id) -> Id {
    if let Some(uid) = state().selected_audio_uid.as_deref() {
        if let Some(index) = index_for_uid(state().audio_devices, uid) {
            return array_item(state().audio_devices, index);
        }
    }
    automatic_audio_device(video_device)
}

unsafe fn error_description(error: Id) -> String {
    if error.is_null() {
        "The source could not be opened".to_owned()
    } else {
        rust_string(msg!(error, "localizedDescription" => Id))
    }
}

unsafe fn clear_inputs() {
    let app_state = state();
    msg!(app_state.session, "beginConfiguration" => ());
    if !app_state.video_input.is_null() {
        msg!(app_state.session, "removeInput:", app_state.video_input; Id => ());
        app_state.video_input = NIL;
    }
    if !app_state.audio_input.is_null() {
        msg!(app_state.session, "removeInput:", app_state.audio_input; Id => ());
        app_state.audio_input = NIL;
    }
    msg!(app_state.session, "commitConfiguration" => ());
}

unsafe fn start_session_once() {
    if state().session_started {
        return;
    }
    state().session_started = true;
    let session_address = retain(state().session) as usize;
    std::thread::spawn(move || {
        let pool = unsafe { objc_autoreleasePoolPush() };
        let session = session_address as Id;
        unsafe {
            msg!(session, "startRunning" => ());
            release(session);
            objc_autoreleasePoolPop(pool);
        }
    });
}

unsafe fn configure_capture(video_device: Id) {
    let video_name = device_property(video_device, "name");
    let video_auth = msg!(
        class(b"AVCaptureDevice\0"),
        "authorizationStatusForMediaType:",
        AVMediaTypeVideo; Id => isize
    );

    if video_auth != AUTH_AUTHORIZED {
        clear_inputs();
        let suffix = if video_auth == AUTH_NOT_DETERMINED {
            "waiting for camera permission"
        } else if video_auth == AUTH_DENIED {
            "camera permission denied"
        } else if video_auth == AUTH_RESTRICTED {
            "camera access restricted"
        } else {
            "camera unavailable"
        };
        set_window_title(&format!("Capture Viewer — {suffix}"));
        return;
    }

    let session = state().session;
    msg!(session, "beginConfiguration" => ());
    if !state().video_input.is_null() {
        msg!(session, "removeInput:", state().video_input; Id => ());
        state().video_input = NIL;
    }
    if !state().audio_input.is_null() {
        msg!(session, "removeInput:", state().audio_input; Id => ());
        state().audio_input = NIL;
    }

    let mut video_error = NIL;
    let video_input = msg!(
        class(b"AVCaptureDeviceInput\0"),
        "deviceInputWithDevice:error:",
        video_device; Id,
        &mut video_error as *mut Id; *mut Id => Id
    );
    if video_input.is_null() || msg!(session, "canAddInput:", video_input; Id => ObjcBool) != YES {
        msg!(session, "commitConfiguration" => ());
        set_window_title(&format!(
            "Capture Viewer — {}",
            error_description(video_error)
        ));
        return;
    }
    msg!(session, "addInput:", video_input; Id => ());
    state().video_input = video_input;

    let audio_auth = msg!(
        class(b"AVCaptureDevice\0"),
        "authorizationStatusForMediaType:",
        AVMediaTypeAudio; Id => isize
    );
    let mut audio_device = NIL;
    if audio_auth == AUTH_AUTHORIZED {
        audio_device = selected_audio_device(video_device);
        if !audio_device.is_null()
            && device_property(audio_device, "uid") != device_property(video_device, "uid")
        {
            let mut audio_error = NIL;
            let audio_input = msg!(
                class(b"AVCaptureDeviceInput\0"),
                "deviceInputWithDevice:error:",
                audio_device; Id,
                &mut audio_error as *mut Id; *mut Id => Id
            );
            if !audio_input.is_null()
                && msg!(session, "canAddInput:", audio_input; Id => ObjcBool) == YES
            {
                msg!(session, "addInput:", audio_input; Id => ());
                state().audio_input = audio_input;
            } else {
                audio_device = NIL;
            }
        }
    }

    msg!(session, "commitConfiguration" => ());
    start_session_once();

    let title = if audio_auth == AUTH_NOT_DETERMINED {
        format!("Capture Viewer — {video_name} — waiting for audio permission")
    } else if audio_auth == AUTH_DENIED || audio_auth == AUTH_RESTRICTED {
        format!("Capture Viewer — {video_name} — audio permission denied")
    } else if audio_device.is_null() {
        format!("Capture Viewer — {video_name} — choose Audio Source")
    } else {
        format!("Capture Viewer — {video_name}")
    };
    set_window_title(&title);
}

unsafe fn configure_selected_video() {
    if let Some(index) = index_for_uid(state().video_devices, &state().selected_video_uid) {
        configure_capture(array_item(state().video_devices, index));
    } else {
        clear_inputs();
        set_window_title("Capture Viewer — No video sources");
    }
}

unsafe fn rebuild_source_menu() {
    let menu = state().source_menu;
    msg!(menu, "removeAllItems" => ());
    let count = array_count(state().video_devices);
    if count == 0 {
        placeholder_menu_item(menu, "No video sources");
        return;
    }

    for index in 0..count {
        let device = array_item(state().video_devices, index);
        let item = new_menu_item(
            &device_property(device, "name"),
            selector(b"selectSource:\0"),
            "",
        );
        msg!(item, "setTarget:", state().controller; Id => ());
        msg!(item, "setTag:", index as isize; isize => ());
        add_menu_item(menu, item);
    }
    refresh_source_checks();
}

unsafe fn rebuild_audio_menu() {
    let menu = state().audio_menu;
    msg!(menu, "removeAllItems" => ());

    let automatic = new_menu_item(
        "Automatic (match video source)",
        selector(b"selectAudioSource:\0"),
        "",
    );
    msg!(automatic, "setTarget:", state().controller; Id => ());
    msg!(automatic, "setTag:", -1isize; isize => ());
    add_menu_item(menu, automatic);
    let separator = msg!(class(b"NSMenuItem\0"), "separatorItem" => Id);
    msg!(menu, "addItem:", separator; Id => ());

    let count = array_count(state().audio_devices);
    for index in 0..count {
        let device = array_item(state().audio_devices, index);
        let item = new_menu_item(
            &device_property(device, "name"),
            selector(b"selectAudioSource:\0"),
            "",
        );
        msg!(item, "setTarget:", state().controller; Id => ());
        msg!(item, "setTag:", index as isize; isize => ());
        add_menu_item(menu, item);
    }
    refresh_audio_checks();
}

unsafe fn reload_devices() {
    let new_video_devices = retain(msg!(
        class(b"AVCaptureDevice\0"),
        "devicesWithMediaType:",
        AVMediaTypeVideo; Id => Id
    ));
    let new_audio_devices = retain(msg!(
        class(b"AVCaptureDevice\0"),
        "devicesWithMediaType:",
        AVMediaTypeAudio; Id => Id
    ));

    release(state().video_devices);
    release(state().audio_devices);
    state().video_devices = new_video_devices;
    state().audio_devices = new_audio_devices;

    if array_count(new_video_devices) == 0 {
        state().selected_video_uid.clear();
    } else if index_for_uid(new_video_devices, &state().selected_video_uid).is_none() {
        state().selected_video_uid = device_property(array_item(new_video_devices, 0), "uid");
    }
    if let Some(uid) = state().selected_audio_uid.as_deref() {
        if index_for_uid(new_audio_devices, uid).is_none() {
            state().selected_audio_uid = None;
        }
    }

    rebuild_source_menu();
    rebuild_audio_menu();
    configure_selected_video();
}

extern "C" fn permission_callback(_block: *mut PermissionBlock, _granted: ObjcBool) {
    unsafe {
        if STATE.is_null() || state().controller.is_null() {
            return;
        }
        msg!(
            state().controller,
            "performSelectorOnMainThread:withObject:waitUntilDone:",
            selector(b"permissionsChanged:\0"); Sel,
            NIL; Id,
            NO; ObjcBool => ()
        );
    }
}

unsafe fn request_permission(media_type: Id) {
    let status = msg!(
        class(b"AVCaptureDevice\0"),
        "authorizationStatusForMediaType:",
        media_type; Id => isize
    );
    if status != AUTH_NOT_DETERMINED {
        return;
    }

    let block = Box::into_raw(Box::new(PermissionBlock {
        isa: ptr::addr_of!(_NSConcreteGlobalBlock),
        flags: 1 << 28, // BLOCK_IS_GLOBAL
        reserved: 0,
        invoke: permission_callback,
        descriptor: &BLOCK_DESCRIPTOR,
    }));
    msg!(
        class(b"AVCaptureDevice\0"),
        "requestAccessForMediaType:completionHandler:",
        media_type; Id,
        block; *mut PermissionBlock => ()
    );
}

extern "C" fn select_source(_controller: Id, _command: Sel, sender: Id) {
    unsafe {
        let index = msg!(sender, "tag" => isize);
        if index < 0 || index as usize >= array_count(state().video_devices) {
            return;
        }
        let device = array_item(state().video_devices, index as usize);
        state().selected_video_uid = device_property(device, "uid");
        refresh_source_checks();
        configure_capture(device);
    }
}

extern "C" fn select_audio_source(_controller: Id, _command: Sel, sender: Id) {
    unsafe {
        let index = msg!(sender, "tag" => isize);
        if index < 0 {
            state().selected_audio_uid = None;
        } else if index as usize >= array_count(state().audio_devices) {
            return;
        } else {
            let device = array_item(state().audio_devices, index as usize);
            state().selected_audio_uid = Some(device_property(device, "uid"));
        }
        refresh_audio_checks();
        configure_selected_video();
    }
}

extern "C" fn refresh_sources(_controller: Id, _command: Sel, _sender: Id) {
    unsafe { reload_devices() }
}

extern "C" fn permissions_changed(_controller: Id, _command: Sel, _sender: Id) {
    unsafe { reload_devices() }
}

extern "C" fn capture_device_changed(_controller: Id, _command: Sel, _notification: Id) {
    unsafe {
        if STATE.is_null() {
            return;
        }
        msg!(
            state().controller,
            "performSelectorOnMainThread:withObject:waitUntilDone:",
            selector(b"permissionsChanged:\0"); Sel,
            NIL; Id,
            NO; ObjcBool => ()
        );
    }
}

extern "C" fn terminate_after_last_window(_controller: Id, _command: Sel, _app: Id) -> ObjcBool {
    YES
}

extern "C" fn application_will_terminate(_controller: Id, _command: Sel, _notification: Id) {
    unsafe {
        if !STATE.is_null() && state().session_started {
            msg!(state().session, "stopRunning" => ());
        }
    }
}

extern "C" fn capture_view_layout(view: Id, _command: Sel) {
    unsafe {
        let mut super_info = ObjcSuper {
            receiver: view,
            super_class: class(b"NSView\0"),
        };
        let super_function: unsafe extern "C" fn(*mut ObjcSuper, Sel) =
            std::mem::transmute(objc_msgSendSuper as *const ());
        super_function(&mut super_info, selector(b"layout\0"));

        if !STATE.is_null() && !state().preview_layer.is_null() {
            let bounds = send_rect(view, b"bounds\0");
            msg!(state().preview_layer, "setFrame:", bounds; Rect => ());
        }
    }
}

unsafe fn register_classes() -> (Class, Class) {
    let controller_class =
        objc_allocateClassPair(class(b"NSObject\0"), c"CaptureViewerController".as_ptr(), 0);
    assert!(!controller_class.is_null());
    class_addMethod(
        controller_class,
        selector(b"selectSource:\0"),
        select_source as *const c_void,
        c"v@:@".as_ptr(),
    );
    class_addMethod(
        controller_class,
        selector(b"selectAudioSource:\0"),
        select_audio_source as *const c_void,
        c"v@:@".as_ptr(),
    );
    class_addMethod(
        controller_class,
        selector(b"refreshSources:\0"),
        refresh_sources as *const c_void,
        c"v@:@".as_ptr(),
    );
    class_addMethod(
        controller_class,
        selector(b"permissionsChanged:\0"),
        permissions_changed as *const c_void,
        c"v@:@".as_ptr(),
    );
    class_addMethod(
        controller_class,
        selector(b"captureDeviceChanged:\0"),
        capture_device_changed as *const c_void,
        c"v@:@".as_ptr(),
    );
    class_addMethod(
        controller_class,
        selector(b"applicationShouldTerminateAfterLastWindowClosed:\0"),
        terminate_after_last_window as *const c_void,
        c"c@:@".as_ptr(),
    );
    class_addMethod(
        controller_class,
        selector(b"applicationWillTerminate:\0"),
        application_will_terminate as *const c_void,
        c"v@:@".as_ptr(),
    );
    objc_registerClassPair(controller_class);

    let view_class = objc_allocateClassPair(class(b"NSView\0"), c"CaptureViewerView".as_ptr(), 0);
    assert!(!view_class.is_null());
    class_addMethod(
        view_class,
        selector(b"layout\0"),
        capture_view_layout as *const c_void,
        c"v@:".as_ptr(),
    );
    objc_registerClassPair(view_class);

    (controller_class, view_class)
}

unsafe fn build_menu(app: Id, controller: Id) -> (Id, Id) {
    let main_menu = msg!(class(b"NSMenu\0"), "new" => Id);

    let app_root = new_menu_item("", ptr::null(), "");
    let app_menu = msg!(class(b"NSMenu\0"), "new" => Id);
    let quit = new_menu_item("Quit Capture Viewer", selector(b"terminate:\0"), "q");
    msg!(quit, "setTarget:", app; Id => ());
    add_menu_item(app_menu, quit);
    msg!(app_root, "setSubmenu:", app_menu; Id => ());
    add_menu_item(main_menu, app_root);
    release(app_menu);

    let file_root = new_menu_item("File", ptr::null(), "");
    let file_menu = msg!(class(b"NSMenu\0"), "new" => Id);
    msg!(file_menu, "setTitle:", ns_string("File"); Id => ());

    let source_item = new_menu_item("Source", ptr::null(), "");
    let source_menu = msg!(class(b"NSMenu\0"), "new" => Id);
    msg!(source_menu, "setTitle:", ns_string("Source"); Id => ());
    msg!(source_item, "setSubmenu:", source_menu; Id => ());
    add_menu_item(file_menu, source_item);

    let audio_item = new_menu_item("Audio Source", ptr::null(), "");
    let audio_menu = msg!(class(b"NSMenu\0"), "new" => Id);
    msg!(audio_menu, "setTitle:", ns_string("Audio Source"); Id => ());
    msg!(audio_item, "setSubmenu:", audio_menu; Id => ());
    add_menu_item(file_menu, audio_item);

    let separator = msg!(class(b"NSMenuItem\0"), "separatorItem" => Id);
    msg!(file_menu, "addItem:", separator; Id => ());
    let refresh = new_menu_item("Refresh Sources", selector(b"refreshSources:\0"), "r");
    msg!(refresh, "setTarget:", controller; Id => ());
    add_menu_item(file_menu, refresh);

    msg!(file_root, "setSubmenu:", file_menu; Id => ());
    add_menu_item(main_menu, file_root);
    release(file_menu);

    msg!(app, "setMainMenu:", main_menu; Id => ());
    release(main_menu);
    (source_menu, audio_menu)
}

unsafe fn build_window(view_class: Class, session: Id) -> (Id, Id) {
    let initial_frame = Rect {
        origin: Point { x: 0.0, y: 0.0 },
        size: Size {
            width: 960.0,
            height: 540.0,
        },
    };
    let style_mask: usize = 1 | 2 | 4 | 8; // titled, closable, miniaturizable, resizable
    let window = msg!(class(b"NSWindow\0"), "alloc" => Id);
    let window = msg!(
        window,
        "initWithContentRect:styleMask:backing:defer:",
        initial_frame; Rect,
        style_mask; usize,
        2usize; usize,
        NO; ObjcBool => Id
    );
    msg!(window, "setTitle:", ns_string("Capture Viewer"); Id => ());
    msg!(window, "setMinSize:", Size { width: 320.0, height: 180.0 }; Size => ());
    msg!(window, "setReleasedWhenClosed:", NO; ObjcBool => ());
    msg!(window, "center" => ());

    let view = msg!(view_class, "alloc" => Id);
    let view = msg!(view, "initWithFrame:", initial_frame; Rect => Id);
    msg!(view, "setWantsLayer:", YES; ObjcBool => ());
    let backing_layer = msg!(view, "layer" => Id);
    let black = msg!(class(b"NSColor\0"), "blackColor" => Id);
    let black_cg_color = msg!(black, "CGColor" => Id);
    msg!(backing_layer, "setBackgroundColor:", black_cg_color; Id => ());

    let preview_layer = msg!(
        class(b"AVCaptureVideoPreviewLayer\0"),
        "layerWithSession:",
        session; Id => Id
    );
    msg!(
        preview_layer,
        "setVideoGravity:",
        AVLayerVideoGravityResizeAspect; Id => ()
    );
    msg!(preview_layer, "setFrame:", initial_frame; Rect => ());
    msg!(backing_layer, "addSublayer:", preview_layer; Id => ());

    msg!(window, "setContentView:", view; Id => ());
    release(view);
    (window, preview_layer)
}

unsafe fn install_device_notifications(controller: Id) {
    let center = msg!(class(b"NSNotificationCenter\0"), "defaultCenter" => Id);
    msg!(
        center,
        "addObserver:selector:name:object:",
        controller; Id,
        selector(b"captureDeviceChanged:\0"); Sel,
        AVCaptureDeviceWasConnectedNotification; Id,
        NIL; Id => ()
    );
    msg!(
        center,
        "addObserver:selector:name:object:",
        controller; Id,
        selector(b"captureDeviceChanged:\0"); Sel,
        AVCaptureDeviceWasDisconnectedNotification; Id,
        NIL; Id => ()
    );
}

fn main() {
    unsafe {
        let pool = objc_autoreleasePoolPush();
        let (controller_class, view_class) = register_classes();
        let app = msg!(class(b"NSApplication\0"), "sharedApplication" => Id);
        msg!(app, "setActivationPolicy:", 0isize; isize => ObjcBool);

        let controller = msg!(controller_class, "new" => Id);
        msg!(app, "setDelegate:", controller; Id => ());

        let session = msg!(class(b"AVCaptureSession\0"), "new" => Id);
        let audio_output = msg!(class(b"AVCaptureAudioPreviewOutput\0"), "new" => Id);
        msg!(audio_output, "setVolume:", 1.0f32; f32 => ());
        if msg!(session, "canAddOutput:", audio_output; Id => ObjcBool) == YES {
            msg!(session, "addOutput:", audio_output; Id => ());
        }

        let (window, preview_layer) = build_window(view_class, session);
        let (source_menu, audio_menu) = build_menu(app, controller);

        STATE = Box::into_raw(Box::new(AppState {
            controller,
            window,
            session,
            preview_layer,
            video_devices: NIL,
            audio_devices: NIL,
            video_input: NIL,
            audio_input: NIL,
            _audio_output: audio_output,
            source_menu,
            audio_menu,
            selected_video_uid: String::new(),
            selected_audio_uid: None,
            session_started: false,
        }));

        install_device_notifications(controller);
        reload_devices();
        request_permission(AVMediaTypeVideo);
        request_permission(AVMediaTypeAudio);

        msg!(window, "makeKeyAndOrderFront:", NIL; Id => ());
        msg!(app, "activateIgnoringOtherApps:", YES; ObjcBool => ());
        objc_autoreleasePoolPop(pool);

        msg!(app, "run" => ());
    }
}

#[cfg(test)]
mod tests {
    use super::normalized_device_name;

    #[test]
    fn pairs_generic_usb_video_and_audio_names() {
        assert_eq!(
            normalized_device_name("USB Video Capture"),
            normalized_device_name("USB Audio")
        );
    }

    #[test]
    fn preserves_identifying_model_text_when_pairing() {
        assert_eq!(
            normalized_device_name("UGREEN 15389 Video"),
            normalized_device_name("UGREEN 15389 Audio Input")
        );
    }
}
