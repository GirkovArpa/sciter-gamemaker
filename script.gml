var foo = external_define(
	"sciter_gamemaker.dll", 
	"foo", 
	dll_cdecl,
	ty_real, 
	1, 
	ty_string
);
var handle = window_handle();
var handle_as_hex_string = string(handle);
external_call(foo, handle_as_hex_string);