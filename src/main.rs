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

macro_rules! impl_get_type {
	($Namespace: ident, $InstanceType: ty, $ClassType: ty) => {
		pub unsafe extern "C" fn get_type() -> glib_ffi::GType {
			static mut TYPE: glib_ffi::GType = gobject_ffi::G_TYPE_INVALID;
			static ONCE: Once = ONCE_INIT;
			ONCE.call_once(|| {
				let type_info = gobject_ffi::GTypeInfo {
					class_size: mem::size_of::<$ClassType>() as u16,
					instance_size: mem::size_of::<$InstanceType>() as u16,
					class_init: None,
					class_finalize: None,
					class_data: ptr::null(),
					instance_init: None,
					base_init: None,
					base_finalize: None,
					n_preallocs: 0,
					value_table: ptr::null()
				};
				let type_name = CString::new(
					{
						let mut type_name = String::from(stringify!($Namespace));
						type_name.push_str(stringify!($InstanceType));
						type_name
					}
				).unwrap();
				TYPE = gobject_ffi::g_type_register_static(
						gobject_ffi::g_object_get_type(),
						type_name.as_ptr(),
						&type_info,
						gobject_ffi::GTypeFlags::empty()
						//0 /*gobject_ffi::GTypeFlags::empty()*/
				);

				/* FIXME: Make this general by adding macro parameters. */
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
	}
}



#[repr(C)]
struct MyListModel {
	pub parent: gobject_ffi::GObject,
	pub items:	Vec<String>
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
					Self::get_type(),
					ptr::null()
				);
			println!("is Self::get_type() {}",
		 		gobject_ffi::g_type_check_instance_is_a(
		 			g_object as *mut _, Self::get_type()
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



	impl_get_type!(Fzn, MyListModel, MyListModelClass);
}
pub unsafe extern "C" fn init_list_model_iface(iface: glib_ffi::gpointer,
											  _data: glib_ffi::gpointer) {
	let iface = &mut *(iface as *mut gio_ffi::GListModelInterface);
	iface.get_item_type = Some(get_item_type);
	iface.get_n_items = Some(get_n_items);
	iface.get_item = Some(get_item);
}



pub unsafe extern "C" fn get_item_type(_self: *mut gio_ffi::GListModel) -> glib_ffi::GType {
	gobject_ffi::G_TYPE_STRING
}

pub unsafe extern "C" fn get_n_items(_self: *mut gio_ffi::GListModel) -> libc::c_uint {
	let this = &*(_self as *mut MyListModel);
	this.items.len() as u32
}

pub unsafe extern "C" fn get_item(_self: *mut gio_ffi::GListModel, index: libc::c_uint) -> glib_ffi::gpointer {
	let this = &*(_self as *mut MyListModel);
	let char_ptr: *mut libc::c_char = this.items[index as usize].to_glib_full();
	char_ptr as glib_ffi::gpointer
}

pub unsafe extern "C" fn create_widget(_item: *mut gobject_ffi::GObject, _data: *mut libc::c_void) -> *mut gtk_ffi::GtkWidget{
/*	let item_char_ptr = _item as *mut libc::c_char;
	let item = CString::from_raw(item_char_ptr).into_string().ok().unwrap();
	println!("Creating {}", item); */
	/*let row = gtk::ListBoxRow::new();
	//let label = gtk::Label::new(Some(&item as &str));
	//row.add(&label);
	row.connect_destroy(move |_| {println!("Destroying123");});
	mem::forget(row.clone());mem::forget(row.clone());
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
    let l = gtk::ListBox::new();
    let lm = MyListModel::new();
 	lm.items.push(String::from("Test1"));
 	lm.items.push(String::from("Test2"));
    unsafe {
    gobject_ffi::g_object_ref(lm as *mut _ as *mut _);
    gtk_ffi::gtk_list_box_bind_model(l.to_glib_none().0,
    								 lm as *mut _ as *mut gio_ffi::GListModel,
    								 Some(create_widget),
    								 ptr::null_mut(),
    								 Some(destroy_notify));
	}
	window.add(&l);
    window.show_all();
	gtk::main();
}
