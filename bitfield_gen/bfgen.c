#include<stdio.h>
#include<stdint.h>
#include<string.h>
#include<stdlib.h>

typedef struct {
    unsigned int can_tag_objects : 1;
    unsigned int can_generate_field_modification_events : 1;
    unsigned int can_generate_field_access_events : 1;
    unsigned int can_get_bytecodes : 1;
    unsigned int can_get_synthetic_attribute : 1;
    unsigned int can_get_owned_monitor_info : 1;
    unsigned int can_get_current_contended_monitor : 1;
    unsigned int can_get_monitor_info : 1;
    unsigned int can_pop_frame : 1;
    unsigned int can_redefine_classes : 1;
    unsigned int can_signal_thread : 1;
    unsigned int can_get_source_file_name : 1;
    unsigned int can_get_line_numbers : 1;
    unsigned int can_get_source_debug_extension : 1;
    unsigned int can_access_local_variables : 1;
    unsigned int can_maintain_original_method_order : 1;
    unsigned int can_generate_single_step_events : 1;
    unsigned int can_generate_exception_events : 1;
    unsigned int can_generate_frame_pop_events : 1;
    unsigned int can_generate_breakpoint_events : 1;
    unsigned int can_suspend : 1;
    unsigned int can_redefine_any_class : 1;
    unsigned int can_get_current_thread_cpu_time : 1;
    unsigned int can_get_thread_cpu_time : 1;
    unsigned int can_generate_method_entry_events : 1;
    unsigned int can_generate_method_exit_events : 1;
    unsigned int can_generate_all_class_hook_events : 1;
    unsigned int can_generate_compiled_method_load_events : 1;
    unsigned int can_generate_monitor_events : 1;
    unsigned int can_generate_vm_object_alloc_events : 1;
    unsigned int can_generate_native_method_bind_events : 1;
    unsigned int can_generate_garbage_collection_events : 1;
    unsigned int can_generate_object_free_events : 1;
    unsigned int can_force_early_return : 1;
    unsigned int can_get_owned_monitor_stack_depth_info : 1;
    unsigned int can_get_constant_pool : 1;
    unsigned int can_set_native_method_prefix : 1;
    unsigned int can_retransform_classes : 1;
    unsigned int can_retransform_any_class : 1;
    unsigned int can_generate_resource_exhaustion_heap_events : 1;
    unsigned int can_generate_resource_exhaustion_threads_events : 1;
    unsigned int can_generate_early_vmstart : 1;
    unsigned int can_generate_early_class_hook_events : 1;
    unsigned int can_generate_sampled_object_alloc_events : 1;
    unsigned int can_support_virtual_threads : 1;
    unsigned int : 3;
    unsigned int : 16;
    unsigned int : 16;
    unsigned int : 16;
    unsigned int : 16;
    unsigned int : 16;
} jvmtiCapabilities;

void dbg(void* cap) {
	uint8_t* n = (uint8_t*) cap;
	for (int i = 0; i < 16; i++) {
		if (n[i] != 0) {
			printf("0x%02X%02X", i, n[i]);
		} 
	}
	printf(";\n");
}

int main() {
	if (sizeof(jvmtiCapabilities) != 16) {
		exit(-1);
	}
	jvmtiCapabilities cap;

	printf("pub const OFFSET_CAN_TAG_OBJECTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_tag_objects = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_FIELD_MODIFICATION_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_field_modification_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_FIELD_ACCESS_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_field_access_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GET_BYTECODES : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_get_bytecodes = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GET_SYNTHETIC_ATTRIBUTE : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_get_synthetic_attribute = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GET_OWNED_MONITOR_INFO : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_get_owned_monitor_info = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GET_CURRENT_CONTENDED_MONITOR : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_get_current_contended_monitor = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GET_MONITOR_INFO : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_get_monitor_info = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_POP_FRAME : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_pop_frame = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_REDEFINE_CLASSES : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_redefine_classes = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_SIGNAL_THREAD : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_signal_thread = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GET_SOURCE_FILE_NAME : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_get_source_file_name = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GET_LINE_NUMBERS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_get_line_numbers = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GET_SOURCE_DEBUG_EXTENSION : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_get_source_debug_extension = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_ACCESS_LOCAL_VARIABLES : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_access_local_variables = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_MAINTAIN_ORIGINAL_METHOD_ORDER : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_maintain_original_method_order = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_SINGLE_STEP_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_single_step_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_EXCEPTION_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_exception_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_FRAME_POP_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_frame_pop_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_BREAKPOINT_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_breakpoint_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_SUSPEND : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_suspend = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_REDEFINE_ANY_CLASS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_redefine_any_class = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GET_CURRENT_THREAD_CPU_TIME : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_get_current_thread_cpu_time = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GET_THREAD_CPU_TIME : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_get_thread_cpu_time = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_METHOD_ENTRY_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_method_entry_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_METHOD_EXIT_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_method_exit_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_ALL_CLASS_HOOK_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_all_class_hook_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_COMPILED_METHOD_LOAD_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_compiled_method_load_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_MONITOR_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_monitor_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_VM_OBJECT_ALLOC_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_vm_object_alloc_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_NATIVE_METHOD_BIND_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_native_method_bind_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_GARBAGE_COLLECTION_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_garbage_collection_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_OBJECT_FREE_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_object_free_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_FORCE_EARLY_RETURN : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_force_early_return = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GET_OWNED_MONITOR_STACK_DEPTH_INFO : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_get_owned_monitor_stack_depth_info = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GET_CONSTANT_POOL : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_get_constant_pool = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_SET_NATIVE_METHOD_PREFIX : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_set_native_method_prefix = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_RETRANSFORM_CLASSES : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_retransform_classes = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_RETRANSFORM_ANY_CLASS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_retransform_any_class = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_RESOURCE_EXHAUSTION_HEAP_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_resource_exhaustion_heap_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_RESOURCE_EXHAUSTION_THREAD_EVENETS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_resource_exhaustion_threads_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_EARLY_VMSTART : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_early_vmstart = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_EARLY_CLASS_HOOK_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_early_class_hook_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_GENERATE_SAMPLED_OBJECT_ALLOC_EVENTS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_generate_sampled_object_alloc_events = 1;
	dbg((void*) &cap);

	printf("pub const OFFSET_CAN_SUPPORT_VIRTUAL_THREADS : usize = ");
	memset((void*) &cap, 0, sizeof(jvmtiCapabilities));
	cap.can_support_virtual_threads = 1;
	dbg((void*) &cap);
	return 0;
}

