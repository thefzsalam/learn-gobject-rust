extern crate gtk;
extern crate glib;
extern crate gio_sys as gio_ffi;
extern crate gtk_sys as gtk_ffi;
extern crate glib_sys as glib_ffi;
extern crate gobject_sys as gobject_ffi;
extern crate libc;
use std::ffi::CString;
use std::ptr;
use std::sync::{Once, ONCE_INIT};
use std::mem;
use glib::translate::*;
use gtk::prelude::*;
use gtk::*;

#[no_mangle]
pub unsafe extern "C" fn my_list_model_get_type() -> glib_ffi::GType {
	static mut TYPE: glib_ffi::GType = gobject_ffi::G_TYPE_INVALID;
	static ONCE: Once = ONCE_INIT;
	ONCE.call_once(|| {
		let type_info = gobject_ffi::GTypeInfo {
			class_size: mem::size_of::<MyListModelClass>() as u16,
			instance_size: mem::size_of::<MyListModel>() as u16,
			class_init: None,
			class_finalize: None,
			class_data: ptr::null(),
			instance_init: None,
			base_init: None,
			base_finalize: None,
			n_preallocs: 0,
			value_table: ptr::null()
		};
		let type_name = CString::new("FznListModel").unwrap();
		TYPE = gobject_ffi::g_type_register_static(
				gobject_ffi::g_object_get_type(),
				type_name.as_ptr(),
				&type_info,
				gobject_ffi::GTypeFlags::empty()
		);
		let list_model_info = gobject_ffi::GInterfaceInfo {
				interface_init: Some(init_list_model_iface),
				interface_finalize: None,
				interface_data: ptr::null_mut()
		};
		gobject_ffi::g_type_add_interface_static(
				TYPE,
				gio_ffi::g_list_model_get_type(),
				&list_model_info
		);
	});
	TYPE
}



#[repr(C)]
struct MyListModel {
	pub parent: gobject_ffi::GObject,
	pub items:	Vec<u8>
}

#[repr(C)]
struct MyListModelClass {
    pub parent_class: gobject_ffi::GObjectClass
}

impl MyListModel {
	pub fn new() -> &'static mut Self {
		unsafe {
			let g_object: *mut gobject_ffi::GObject =
				gobject_ffi::g_object_new(
					my_list_model_get_type(),
					ptr::null()
				);
			println!("is my_list_get_type() {}",
		 		gobject_ffi::g_type_check_instance_is_a(
		 			g_object as *mut _, my_list_model_get_type()
		 		)
			);
			println!("is gio_list_model_get_type() {}",
		 		gobject_ffi::g_type_check_instance_is_a(
		 			g_object as *mut _, gio_ffi::g_list_model_get_type()
		 		)
			);
			let my_list_model = &mut *(g_object as *mut MyListModel);
			ptr::write(&mut my_list_model.items as *mut _, Vec::new());
			my_list_model
		}
	}
}

pub unsafe extern "C" fn init_list_model_iface(iface: glib_ffi::gpointer,
											  _data: glib_ffi::gpointer) {
	let iface = &mut *(iface as *mut gio_ffi::GListModelInterface);
	iface.get_item_type = Some(get_item_type);
	iface.get_n_items = Some(get_n_items);
	iface.get_item = Some(get_item);
}

pub unsafe extern "C" fn get_item_type(_self: *mut gio_ffi::GListModel) -> glib_ffi::GType {
	println!("get_item_type {}", gobject_ffi::G_TYPE_UINT);
	gobject_ffi::G_TYPE_UINT
}

pub unsafe extern "C" fn get_n_items(_self: *mut gio_ffi::GListModel) -> libc::c_uint {
	let this = &*(_self as *mut MyListModel);
	println!("Length is {}", this.items.len());
	this.items.len() as u32
}

pub unsafe extern "C" fn get_item(_self: *mut gio_ffi::GListModel, index: libc::c_uint) -> glib_ffi::gpointer {
	let this = &*(_self as *mut MyListModel);
	let ptr: *mut libc::uint8_t = &(this.items[index as usize]) as *const _ as *mut _;
	println!("get_item {}", *ptr);
	ptr as glib_ffi::gpointer
}

pub unsafe extern "C" fn create_widget(_item: *mut gobject_ffi::GObject, _data: *mut libc::c_void) -> *mut gtk_ffi::GtkWidget{
	let item_ptr = _item as *mut libc::uint8_t;
	println!("Creating {}", *item_ptr);
	/*let row = gtk::ListBoxRow::new();
	let label = gtk::Label::new(Some(&item as &str));
	row.add(&label);
	row.connect_destroy(move |_| {println!("Destroying Row");});
	row.to_glib_full()*/
	gtk_ffi::gtk_list_box_row_new()
}

pub unsafe extern "C" fn destroy_notify(_data: *mut libc::c_void) {
	println!("Destroy Notify");
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let window = gtk::Window::new(WindowType::Toplevel);
    let listbox = gtk::ListBox::new();
    let listmodel = MyListModel::new();

    /* commenting out this line prevents SISSEGV */
 	listmodel.items.push(10);

    unsafe {
    gtk_ffi::gtk_list_box_bind_model(
		listbox.to_glib_none().0,
		listmodel as *mut _ as *mut gio_ffi::GListModel,
		Some(create_widget),
		ptr::null_mut(),
		Some(destroy_notify));
	}
	window.add(&listbox);
    window.show_all();
	gtk::main();
}
