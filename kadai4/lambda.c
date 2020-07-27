#define pr_fmt(fmt) KBUILD_MODNAME ": " fmt

#include <linux/kallsyms.h>
#include <linux/module.h>
#include <linux/fs.h>
#include <linux/types.h>
#include <linux/kernel.h>
#include <linux/uaccess.h>
#include <asm/uaccess.h>
#include <linux/syscalls.h>
#include <linux/slab.h>
#include <linux/mm.h>
#include "lambda.h"

#define DRIVER_MEJOR 63
#define DRIVER_NAME "lambda"

struct JsonValue *out=NULL, *read_hook=NULL;

struct List {
	struct JsonValue json;
	struct List *next;
};

struct KeyValueList {
	struct JsonValue json;
	char *key;
	struct KeyValueList *next;
};

ParseResult gen_error(int pos) {
	ParseResult result;
	result.type = ERROR;
	result.pos = pos;
	return result;
}

int is_white(char c) {
	return c == ' ' || c == '\t' || c == '\n' || c == '\r';
}

int is_digit(char c) {
	return c >= '0' && c <= '9';
}

ParseResult parse_impl(int count, const char *input) {
	long long origin = (long long)input;
	int is_true = 1;
	char true_str[] = "true";
	int is_false = 1;
	char false_str[] = "false";
	ParseResult result;
	while(is_white(*input) || count <= 0) {
		++input;
		--count;
	}
	if (count == 0) {
		return gen_error((long long)input);
	}
	if (*input == '"') {
		int i, size=0;
		for (i=1;; ++i, ++size) {
			if (i >= count) return gen_error((long long)input);
			else if (input[i] == '\\' && input[i+1] == '"') {
				if (i+2 >= count) return gen_error((long long)input);
				++i;
			}
			else if (input[i] == '"') {
				char *buf = (char*)kmalloc(size+1, GFP_KERNEL);
				struct JsonValue value;
				int j, dest;
				buf[size] = '\0';
				for (j=1, dest=0;j<i; ++j, ++dest) {
					if (input[j] == '\\' && input[j+1] == '"') {
						buf[dest] = input[j+1];
						++j;
					}
					else {
						buf[dest] = input[j];
					}
				}
				result.type = SUCCESS;
				value.type = STRING;
				value.string.buf = buf;
				value.string.len = size;
				result.value = value;
				result.used = (long long)input - origin + i + 1;
				return result;
			}
		}
		return gen_error((long long)input);
	}
	if (*input == '-' || is_digit(*input)) {
		int sign = 1;
		int base=0;
		int i=0;
		struct JsonValue value;
		if (count < 1) return gen_error((long long)input);
		if (*input == '-') {
			sign = -1;
			input++;
			count--;
		}
		while (i < count && is_digit(input[i])) {
			base *= 10;
			base += input[i++] - '0';
		}
		if (i == 0) return gen_error((long long)input);
		result.type = SUCCESS;
		value.integer = base * sign;
		value.type = INTEGER;
		result.value = value;
		result.used = i + (long long)input - origin;
		return result;
	}
	if (*input == '[') {
		struct List *list = NULL;
		struct List *l1 = list;
		int cnt = 0;
		int idx = cnt;
		struct JsonValue *arr;
		++input; --count;
		if (count < 0) return gen_error((long long)input);
		while (is_white(*input)) {
			if (count < 0) return gen_error((long long)input);
			++input; --count;
		}
		if (count < 0) return gen_error((long long)input);
		if (*input == ']') {
			result.type = SUCCESS;
			result.used = (long long)input - (long long)origin + 1;
			result.value.type = ARRAY;
			result.value.arrary.len = 0;
			result.value.arrary.arr = NULL;
			return result;
		}

		for (;;) {
			ParseResult result = parse_impl(count, input);
			struct List *next;
			if (result.type == ERROR) {
				result.type = ERROR;
				result.pos = (long long)input;
				return result;
			}
			input += result.used;
			count -= result.used;
			next = kmalloc(sizeof(struct List), GFP_KERNEL);
			next->next = list;
			next->json = result.value;
			list = next;

			if (count < 1) return gen_error((long long)input);
			while (is_white(*input)) {
				if (count <= 1) return gen_error((long long)input);
				++input; --count;
			}

			if (*input == ',') {
				if (count < 1) return gen_error((long long)input);
				++input; --count;
			}
			else if (*input == ']') {
				if (count < 1) return gen_error((long long)input);
				break;
			}
		}
		l1 = list;
		while(l1 != NULL) {
			++cnt;
			l1 = l1->next;
		}
		idx = cnt;
		arr = kmalloc(sizeof(struct JsonValue) * cnt, GFP_KERNEL);
		while(list != NULL) {
			arr[--idx] = list->json;
			list = list->next;
		}
		result.type = SUCCESS;
		result.used = (long long)input - origin + 1;
		result.value.type = ARRAY;
		result.value.arrary.arr = arr;
		result.value.arrary.len = cnt;
		return result;
	}
	if (*input == '{') {
		int cnt = 0;
		struct KeyValueList *l1, *list=NULL;
		int idx;
		struct Pair *pairs;
		++input; --count;
		if (count <= 0) return gen_error((long long)input);
		while (is_white(*input)) {
			if (count < 0) return gen_error((long long)input);
			++input; --count;
		}
		if (count <= 0) return gen_error((long long)input);
		if (*input == '}') {
			result.type = SUCCESS;
			result.used = (long long)input - (long long)origin + 1;
			result.value.type = OBJECT;
			result.value.arrary.len = 0;
			result.value.arrary.arr = NULL;
			return result;
		}

		for (;;) {
			ParseResult result = parse_impl(count, input);
			struct KeyValueList *next;
			struct String key;
			if (result.type == ERROR || result.value.type != STRING) {
				result.type = ERROR;
				result.pos = (long long)input;
				return result;
			}
			key = result.value.string;
			input += result.used;
			count -= result.used;
			while (is_white(*input)) {
				if (count < 0) return gen_error((long long)input);
				++input;
				--count;
			}
			if (*input != ':') return gen_error((long long)input);
			++input;
			--count;
			result = parse_impl(count, input);
			if (result.type == ERROR) {
				result.type = ERROR;
				result.pos = (long long)input;
				return result;
			}
			input += result.used;
			count -= result.used;
			next = kmalloc(sizeof(struct KeyValueList), GFP_KERNEL);
			next->next = list;
			next->json = result.value;
			next->key = key.buf;
			list = next;

			if (count < 1) return gen_error((long long)input);
			while (is_white(*input)) {
				if (count < 1) return gen_error((long long)input);
				++input; --count;
			}

			if (*input == ',') {
				if (count < 1) return gen_error((long long)input);
				++input; --count;
			}
			else if (*input == '}') {
				if (count < 1) return gen_error((long long)input);
				break;
			}
		}
		l1 = list;
		while(l1 != NULL) {
			++cnt;
			l1 = l1->next;
		}
		pairs = kmalloc(sizeof(struct Pair) * cnt, GFP_KERNEL);
		idx = cnt;
		while(list != NULL) {
			pairs[--idx] = (struct Pair){ list->key, list->json };
			list = list->next;
		}
		result.type = SUCCESS;
		result.used = (long long)input - origin + 1;
		result.value.type = OBJECT;
		result.value.pairs.pairs = pairs;
		result.value.pairs.len = cnt;
		result.value.pairs.mem_len = cnt;
		return result;
	}
	if (count >= 4) {
		int i;
		for (i = 0; i < 4; ++i) {
			is_true &= true_str[i] == input[i];
		}
	}
	if (count >= 5) {
		int i;
		for (i = 0; i < 5; ++i) {
			is_false &= false_str[i] == input[i];
		}
	}
	if (is_true || is_false) {
		struct JsonValue value;
		result.type = SUCCESS;
		value.type = BOOLEAN;
		value.boolean = is_true;
		result.value = value;
		result.used = (long long)input - origin + is_true ? 4 : 5;
		return result;
	}
	return gen_error((long long)input);
}

ParseResult parse(const char *input, int count) {
	ParseResult result;
	result = parse_impl(count, input);
	if (result.type == ERROR) {
		result.pos -= (long long)input;
	}
	return result;
}

int stringify_impl(char *buf, int buf_size, JSONValue json) {
	if (buf_size < 1) return -1;
	if (json.type == STRING) {
		int cursor = 1, i;
		buf[0] = '"';
		for (i=0; i<json.string.len; ++i) {
			if (json.string.buf[i] == '"') {
				if (cursor+2 >= buf_size) return -1;
				buf[cursor++] = '\\';
				buf[cursor++] = '"';
			}
			else if (json.string.buf[i] == '\\') {
				if (cursor+2 >= buf_size) return -1;
				buf[cursor++] = '\\';
				buf[cursor++] = '\\';
			}
			else {
				if (cursor+1 >= buf_size) return -1;
				buf[cursor++] = json.string.buf[i];
			}
		}
		if (cursor+1 >= buf_size) return -1;
		buf[cursor++] = '\"';
		return cursor;
	}
	else if (json.type == INTEGER) {
		int i=0;
		int base = 1;
		int abs = json.integer > 0 ? json.integer : -json.integer;
		char tmp[1024];
		int j=0, k;
		if (i >= buf_size-1) return -1;
		if (json.integer < 0)
			buf[i++] = '-';
		while(abs / base > 0) {
			tmp[j++] = (abs / base) % 10 + '0';
			if (j+i >= buf_size) return -1;
			base *= 10;
		}
		for(k=0;k<j;++k) {
			buf[i+k] = tmp[j-k-1];
		}
		return j+i;
	}
	else if (json.type == ARRAY) {
		int offset = 1;
		int i;
		if (buf_size < 1) return -1;
		buf[0] = '[';
		for (i=0; i<json.arrary.len; ++i) {
			int s;
			if (buf_size - offset < 1) return -1;
			s = stringify_impl(buf + offset, buf_size-1, json.arrary.arr[i]);
			if (s == -1) return -1;
			offset += s;
			if (i < json.arrary.len-1) {
				if (buf_size - offset < 2) return -1;
				buf[offset++] = ',';
			}
		}
		if (buf_size - offset < 1) return -1;
		buf[offset++] = ']';
		return offset;
	}
	else if (json.type == OBJECT ) {
		int offset = 1;
		int i, j;
		if (buf_size < 1) return -1;
		buf[0] = '{';
		for (i=0; i<json.pairs.len; ++i) {
			int s;
			buf[offset] = '"';
			for (j=0; json.pairs.pairs[i].key[j] != '\0'; ++j) {
				buf[++offset] = json.pairs.pairs[i].key[j];
			}
			if (buf_size - offset < 2) return -1;
			buf[++offset] = '"';
			buf[++offset] = ':';
			++offset;
			s = stringify_impl(buf + offset, buf_size-offset, json.pairs.pairs[i].value);
			if (s == -1) return -1;
			offset += s;
			if (i < json.arrary.len-1) {
				if (buf_size - offset < 2) return -1;
				buf[offset++] = ',';
			}
		}
		if (buf_size - offset < 1) return -1;
		buf[offset++] = '}';
		return offset;
	}
	else if (json.type == BOOLEAN) {
		if (json.boolean) {
			int i;
			char src[] = "true";
			if (buf_size < sizeof(src)) return -1;
			for (i = 0; i < sizeof(src); ++i) {
				buf[i] = src[i];
			}
			return sizeof(src)-1;
		}
		else {
			char src[] = "false";
			int i;
			if (buf_size < sizeof(src)) return -1;
			for (i = 0; i < sizeof(src); ++i) {
				buf[i] = src[i];
			}
			return sizeof(src)-1;
		}
	}
	else {
		return 0;
	}
}

int stringify(char *buf, int buf_size, JSONValue json) {
	int len = stringify_impl(buf, buf_size-1, json);
	if (len >= 0) {
		buf[len] = '\0';
		return len+1;
	}
	return -1;
}


struct Path {
	int len;
	struct PathElem *path;
};

struct PathElem {
	int is_ref;
	union {
		struct Path path;
		char *name;
	};
};

struct PathElemList {
	struct PathElemList *next;
	struct PathElem e;
};

struct JsonValue* empty_object(void) {
	struct JsonValue *json = (struct JsonValue*)kmalloc(sizeof(struct JsonValue),GFP_KERNEL);
	json->type = OBJECT;
	json->pairs.len = 0;
	json->pairs.mem_len = 0;
	return json;
}

int parse_path(struct Path *path_buf, char *buf, int count) {
	int len=0, idx=0, begin=0;
	int j;
	struct PathElemList *list = NULL;
	struct PathElem *elems;
	while (idx < count) {
		if (buf[idx] == '.') {
			char *name = (char*)kmalloc(idx-begin+1, GFP_KERNEL);
			struct PathElemList *new;
			for (j=0;j<idx-begin; ++j) {
				name[j] = buf[begin+j];
			}
			name[idx-begin] = '\0';
			new = (struct PathElemList*)kmalloc(sizeof(struct PathElemList), GFP_KERNEL);
			new->e.is_ref = 0;
			new->e.name = name;
			new->next = list;
			list = new;
			begin = idx+1;
			++len;
		}
		else if (buf[idx] == '[') {
			int end = idx+1;
			int paren_count = 0;
			int succ=0;
			char *name;
			struct Path sub;
			struct PathElemList *new;
			while (end<count) {
				if (buf[end] == ']') {
					if (paren_count == 0) {
						succ=1;
						break;
					}
					else {
						--paren_count;
					}
				}
				else if (buf[end] == '[') {
					++paren_count;
				}
				++end;
			}
			if (!succ) return 0;
			if (!parse_path(&sub, buf+idx+1, end-idx-1))
				return 0;
			new = (struct PathElemList*)kmalloc(sizeof(struct PathElemList), GFP_KERNEL);
			new->e.is_ref = 0;
			name = (char*)kmalloc(idx-begin+1, GFP_KERNEL);
			for (j=0;j<idx-begin; ++j) {
				name[j] = buf[begin+j];
			}
			new->e.name = name;
			new->next = list;
			list = new;
			new = (struct PathElemList*)kmalloc(sizeof(struct PathElemList), GFP_KERNEL);
			new->e.is_ref = 1;
			new->e.path = sub;
			new->next = list;
			list = new;
			begin = end;
			idx = end;
			len+=2;
		}
		++idx;
	}
	if (count-begin > 1) {
		struct PathElemList *new;
		char *name = (char*)kmalloc(count-begin+1, GFP_KERNEL);
		for (j=0;j<count-begin; ++j) {
			name[j] = buf[begin+j];
		}
		name[idx-begin] = '\0';
		new = (struct PathElemList*)kmalloc(sizeof(struct PathElemList), GFP_KERNEL);
		new->e.is_ref = 0;
		new->e.name = name;
		new->next = list;
		list = new;
		++len;
	}
	elems = (struct PathElem*)kmalloc(len*sizeof(struct PathElem), GFP_KERNEL);
	for (j=0; j<len; ++j) {
		elems[len-j-1] = list->e;
		list = list->next;
	}
	path_buf->len = len;
	path_buf->path = elems;
	return 1;
}

int str_same(char *a, char *b) {
	int idx = 0;
	for (;;) {
		if (a[idx] == '\0' && b[idx] == '\0') {
			return 1;
		}
		if (a[idx] !=  b[idx]) {
			return 0;
		}
		++idx;
	}
}

int str_len(char* a) {
	int idx=0;
	while (a[idx++] != '\0') {}
	return idx;
}

struct JsonValue *get_value(struct JsonValue *json, struct PathElem *path, int len);

char* resolve_path_elem(struct JsonValue *json, struct PathElem path) {
	if (path.is_ref) {
		struct JsonValue* s;
		if (path.path.len <= 0) return NULL;
		s = get_value(json, path.path.path, path.path.len);
		if (s == NULL || s->type != STRING) return NULL;
		return s->string.buf;
	}
	else {
		return path.name;
	}
}

struct JsonValue *get_value_sys_time(int len) {
	struct JsonValue *time;
	ktime_t sys_time = ktime_get();
	if (len != 0) return NULL;
	time = (struct JsonValue*)kmalloc(sizeof(struct JsonValue), GFP_KERNEL);
	time->type = INTEGER;
	time->integer = (int)sys_time;
	return NULL;
}

struct JsonValue *access(struct JsonValue *json, char* buf) {
	int i;
	for (i=0; i<json->pairs.len; ++i) {
		if (str_same(json->pairs.pairs[i].key, buf)) {
			return &json->pairs.pairs[i].value;
		}
	}
	return NULL;
}

struct JsonValue *get_out_value(struct JsonValue *json, struct PathElem *path, int len, int create_elems) {
	char *head;
	int head_len;
	char *key_buf;
	int i;
	struct JsonValue value;
	if (len <= 0) return json;
	head = resolve_path_elem(json, *path);
	if (head == NULL) return NULL;
	if (json->type != OBJECT) return NULL;
	for (i=0; i<json->pairs.len; ++i) {
		if (str_same(json->pairs.pairs[i].key, head)) {
			return get_out_value(&json->pairs.pairs[i].value, path+1, len-1, create_elems);
		}
	}
	if (!create_elems) return NULL;
	if (json->pairs.len+1 > json->pairs.mem_len) {
		int i;
		struct Pair *pairs = (struct Pair*)kmalloc(sizeof(struct Pair)*json->pairs.mem_len*2+1, GFP_KERNEL);
		if (pairs == NULL) return NULL;
		for (i=0; i<json->pairs.len; ++i) {
			pairs[i] = json->pairs.pairs[i];
		}
		if(json->pairs.mem_len > 0)
			kfree(json->pairs.pairs);
		json->pairs.pairs = pairs;
		json->pairs.mem_len = json->pairs.mem_len*2+1;
	}
	value.type = OBJECT;
	value.pairs.pairs = (struct Pair*)kmalloc(sizeof(struct Pair), GFP_KERNEL);
	value.pairs.len = 0;
	value.pairs.mem_len = 0;

	head_len = str_len(head);
	key_buf = (char*)kmalloc(head_len, GFP_KERNEL);
	for (i = 0; i<head_len; ++i) {
		key_buf[i] = head[i];
	}
	json->pairs.pairs[json->pairs.len].key = key_buf;
	json->pairs.pairs[json->pairs.len].value = value;
	json->pairs.len++;
	return get_out_value(&json->pairs.pairs[json->pairs.len-1].value, path+1, len-1, create_elems);
}

struct JsonValue* get_value_sys(struct JsonValue *json, struct PathElem *path, int len) {
	char *head;
	if (len <= 0) return NULL;
	head = resolve_path_elem(json, *path);
	if (head == NULL) return NULL;
	if (str_same(head, "time"))
		return get_value_sys_time(len-1);
	return NULL;
}

struct JsonValue* get_value(struct JsonValue *json, struct PathElem *path, int len) {
	char *head;
	if (len <= 0) return NULL;
	head = resolve_path_elem(json, *path);
	if (head == NULL)
	if (head == NULL) return NULL;
	if (str_same(head, "out"))
		return get_out_value(json, path+1, len-1, 0);
	if (str_same(head, "sys"))
		return get_value_sys(json, path+1, len-1);
	return NULL;
}

void sweep(struct JsonValue *json) {
	int i;
	switch (json->type) {
		case INTEGER:
			return;
		case BOOLEAN:
			return;
		case STRING:
			return;
		case OBJECT:
			for (i=0; i<json->pairs.len; ++i) {
				kfree(json->pairs.pairs[i].key);
				sweep(&json->pairs.pairs[i].value);
			}
			return;
		case ARRAY:
			for (i=0; i<json->arrary.len; ++i) {
				sweep(json->arrary.arr + i);
			}
			return;
	}
}

struct JsonValue* eval(struct JsonValue* out, struct JsonValue *root) {
	if (root->type == OBJECT) {
		struct JsonValue *type = access(root, "type");
		if (type == NULL || type->type != STRING) return NULL;
		if (str_same(type->string.buf, "op")) {
			struct JsonValue *op = access(root, "op");
			struct JsonValue *lhr, *rhr;
			if (op == NULL || op->type != STRING) return NULL;

			lhr = access(root, "lhr");
			if (lhr == NULL) return NULL;

			rhr = access(root, "rhr");
			if (rhr == NULL) return NULL;

			if (str_same(op->string.buf, "sub")) {
				struct JsonValue *lhr_evaluated = eval(out, lhr);
				struct JsonValue *rhr_evaluated = eval(out, rhr);
				struct JsonValue *json;
				if (lhr_evaluated == NULL ||
					lhr_evaluated->type != INTEGER ||
					rhr_evaluated == NULL ||
					rhr_evaluated->type != INTEGER)
					return NULL;
				json = (struct JsonValue*)kmalloc(sizeof(struct JsonValue), GFP_KERNEL);
				json->type = INTEGER;
				json->integer = lhr_evaluated->integer - rhr_evaluated->integer;
				return json;
			}
		}
		else if (str_same(type->string.buf, "ref")) {
			struct JsonValue *name = access(root, "name");
			struct Path path;
			if (name == NULL || name->type != STRING) return NULL;
			if (!parse_path(&path, name->string.buf, name->string.len))
				return NULL;
			return get_value(out, path.path, path.len);
		}
	}
	return root;
}

int exec(struct JsonValue* out, struct JsonValue *json) {
	struct JsonValue *type = access(json, "type");
	if (type == NULL || type->type != STRING) return 0;
	if (str_same(type->string.buf, "assign")) {
		struct JsonValue *target = access(json, "target");
		struct JsonValue *value = access(json, "value");
		struct JsonValue *target_ptr, *json;
		struct Path path;
		if (value == NULL || target == NULL || target->type != STRING) return 0;
		if (!parse_path(&path, target->string.buf, target->string.len))
			return 0;
		if (path.len == 0 || path.path->is_ref || !str_same(path.path->name, "out")) return 0;
		target_ptr = get_out_value(out, path.path+1, path.len-1, 1);
		if (target_ptr == NULL) return 0;
		sweep(target_ptr);
		json = eval(out, value);
		if (json == NULL) return 0;
		*target_ptr = *json;
		sweep(json);
		return 1;
	}
	return 0;
}

int set_hook(struct JsonValue* out, struct JsonValue* json) {
	return 0;
}

int load(struct JsonValue* out, struct JsonValue *json) {
	struct JsonValue *type = access(json, "type");
	if (type == NULL || type->type != STRING) return 0;
	if (str_same(type->string.buf, "probe")) {
		struct JsonValue *hooks = access(json, "hooks");
		int i;
		if (type == NULL || hooks->type != ARRAY) return 0;
		for (i=0; i<hooks->arrary.len; ++i) {
			if (!set_hook(out, &hooks->arrary.arr[i]))
				return 0;
		}
		return 1;
	}
	return 0;
}


static void **syscall_table;
asmlinkage long (*orig_read)(int magic1, int magic2, unsigned int cmd, void __user *arg);
asmlinkage long syscall_replace_read(int magic1, int magic2, unsigned int cmd, void __user *arg) {
	if (read_hook != NULL) {
		exec(out, read_hook);
	}
	return (*orig_read)(magic1, magic2, cmd, arg);
}

static void save_original_syscall_address(void) {
	pr_info("read original address 0x%p + 0x%d\n", syscall_table, __NR_read);
	//orig_read = syscall_table[__NR_read];
}

static void change_page_attr_to_rw(pte_t *pte) {
	set_pte_atomic(pte, pte_mkwrite(*pte));
}

static void change_page_attr_to_ro(pte_t *pte) {
	set_pte_atomic(pte, pte_clear_flags(*pte, _PAGE_RW));
}

static void replace_syscall(void *new) {
	unsigned int level = 0;
	pte_t *pte;
	/*
	pte = lookup_address((unsigned long) syscall_table, &level);
	change_page_attr_to_rw(pte);
	syscall_table[__NR_read] = syscall_replace_read;
	change_page_attr_to_ro(pte);*/
}

static int syscall_replace_init(void) {
	syscall_table = (void**)0xffffffff81e00280;
	pr_info("sys_call_table address is 0x%p\n", syscall_table);
	save_original_syscall_address();
	replace_syscall(syscall_replace_read);
	pr_info("system call replaced\n");
	return 0;
}

static void syscall_replace_cleanup(void) {
	pr_info("cleanup");
	if (orig_read)
		replace_syscall(orig_read);
}

char srcbuf[1024];
size_t len;

struct runtime_info_t {
	int tag;
	struct JsonValue json;
};

static int lambda_open(struct inode *inode, struct file *file) {
	struct runtime_info_t *info = kmalloc(sizeof(struct runtime_info_t), GFP_KERNEL);
	info->tag = 0;
	printk("lambda open\n");
	file->private_data = (void*)info;
	return 0;
}

static int lambda_release(struct inode *inode, struct file *file) {
	printk("lambda close\n");
	return 0;
}

static ssize_t lambda_write(struct file *file, const char __user *buf, size_t count , loff_t *f_pos) {
	struct runtime_info_t *info = file->private_data;
	ParseResult result;
	printk("lambda write\n");
	if (!access_ok(buf, count)) {
		return 0;
	}
	printk("parse start %s %ld\n", buf, count);
	result = parse(buf, count);
	if (result.type != SUCCESS) {
		printk("syntax error");
		return 0;
	}
	printk("parse success %d\n", result.value.type);
	info->json = result.value;
	info->tag = 42;
	return count;
}

static ssize_t lambda_read(struct file *file, char __user *buf, size_t count, loff_t *f_pos) {
	struct runtime_info_t *info = file->private_data;
	long long len;
	printk("lambda read\n");
	if (!access_ok(buf, count)) {
		return 0;
	}
	len = stringify(buf, count, info->json);
	printk("stringified %lld %d %d\n", len, info->json.type, info->tag);
	if (len < 0) return 0;
	return 0;
}

struct file_operations s_lambda_fops = {
	.open		= lambda_open,
	.release	= lambda_release,
	.read		= lambda_read,
	.write		= lambda_write,
};

static int lambda_init(void) {
	printk("Hello lambda\n");
	register_chrdev(DRIVER_MEJOR, DRIVER_NAME, &s_lambda_fops);
	syscall_replace_init();
	return 0;
}

static void lambda_exit(void) {
	printk("Goodbye lambda\n");
	syscall_replace_cleanup();
	unregister_chrdev(DRIVER_MEJOR, DRIVER_NAME);
}

module_init(lambda_init);
module_exit(lambda_exit);

MODULE_DESCRIPTION("lambda");
MODULE_AUTHOR("namachan10777");
MODULE_LICENSE("GPL");


