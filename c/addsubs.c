#include <asm-generic/errno-base.h>
#include <dirent.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <sys/stat.h> // mkdir
#include <errno.h> // get libc error

typedef char *string;

enum ProgramError {
	LANGUAGE_ERR = 1,
	DIR_ERR,
	NO_FILE_ERR,
	MISMATCH_ERR,
	RESPONSE_ERR,
	USER_CANCEL,
	OUTPUT_MKDIR_ERR,
	SNPRINTF_ERR,
	SPRINTF_ERR,
	ARG_COUNT_ERR,
	THREAD_CREATE_ERR,
	THREAD_JOIN_ERR,
};

string program_error_msg(int err) {
	switch (err) {
		case LANGUAGE_ERR:
			return "error: language not supported.";
		case DIR_ERR:
			return "error: that is not a directory.";
		case NO_FILE_ERR:
			return "error: either there are no video files or sub files that meet the file format specified.";
		case MISMATCH_ERR:
			return "error: no equal ammount of video and sub files.";
		case RESPONSE_ERR:
			return "error: response was too long?";
		case USER_CANCEL:
		case OUTPUT_MKDIR_ERR:
			return "error: mkdir error out unexepectedly. Check permissios on the directory.";
		case SNPRINTF_ERR:
			return "error: encoding failed or command line string too long.";
		case SPRINTF_ERR:
			return "error: sprintf encoding failed";
		case ARG_COUNT_ERR:
			return "error: wrong amount of arguments.";
		case THREAD_CREATE_ERR:
			return "error: thread create failed.";
		case THREAD_JOIN_ERR:
			return "error: pthread join failed.";
		default:
			return "undefined error";
	}
}

typedef struct	{
	string *ptr;
	size_t len;
	size_t size;
} StrVec;

void append_strvec(StrVec *v, const string s) {
	v->len += 1;
	if (v->len > v->size) {
		v->size *= 2;
		v->ptr = realloc(v->ptr, v->size * sizeof(string));
	}
	v->ptr[v->len] = strdup(s);
}

StrVec new_strvec(size_t size) {
	StrVec ret;
	ret.size = size;
	ret.len = 0;
	ret.ptr = calloc(size, sizeof(string)); // check if need to calloc internals;
	return ret;
}

void free_strvec(StrVec* v) {
	for (int i = 0; i <= v->len; i++) {
		free(v->ptr[i]);
	}
	v->len = v->size = 0;
	v->ptr = NULL;
}

string langs(string lang) {
	if (strcmp(lang, "jpn") == 0) {
		return "Japanese";
	} else {
		return NULL;
	}
}

int compare_str(const void *a, const void *b) {
	return strcmp((string)a, (string)b);
}

void* run_cmd(void* input) {
	int ret = system((string)input);
	return NULL;
}

// TODO testing
int addsubs(const string dir, const string videoformat, const string subformat, const string lang) {
	string language = langs(lang);
	if (language == NULL) {
		puts(program_error_msg(LANGUAGE_ERR));
		return LANGUAGE_ERR;
	}

	StrVec videofiles = new_strvec(12);
	StrVec subfiles = new_strvec(12);

	DIR *folder = opendir(dir);
	if (folder == NULL) {
		puts(program_error_msg(DIR_ERR));
		return DIR_ERR;
	}

	struct dirent *entry;
	while ((entry = readdir(folder))) {
		if (entry->d_type == DT_REG) {
			if (strstr(entry->d_name, videoformat) != NULL) {
				append_strvec(&videofiles, entry->d_name);
			} else if (strstr(entry->d_name, subformat) != NULL) {
				append_strvec(&subfiles, entry->d_name);
			}
		}
	}
	closedir(folder);

	if (videofiles.len == 0 || subfiles.len == 0) {
		puts(program_error_msg(NO_FILE_ERR));
		return NO_FILE_ERR;
	}
	if (videofiles.len != subfiles.len) {
		puts(program_error_msg(MISMATCH_ERR));
		return MISMATCH_ERR;
	}

	qsort(&videofiles, videofiles.len, sizeof(string), compare_str);
	qsort(&subfiles, subfiles.len, sizeof(string), compare_str);

	puts("Are these pairs correct? (Y/n):");
	char response[10];
	if (scanf("%s", response) < 0) {
		puts(program_error_msg(RESPONSE_ERR));
		return RESPONSE_ERR;
	}
	if (strstr(response, "n") != NULL) {return USER_CANCEL;}

	const string os = "/output";
	string output_dir = malloc(strlen(dir) + strlen(os) +  1);
	strcpy(output_dir, dir);
	strcat(output_dir, os);
	int ret = mkdir(output_dir, 0777);
	if (ret != 0 && errno != EEXIST) {
		puts(program_error_msg(OUTPUT_MKDIR_ERR));
		return OUTPUT_MKDIR_ERR;
	}

	// TODO test loop and join
	pthread_t thread_id;
	for (int i = 0; i < subfiles.len; i++) {
		string sub = subfiles.ptr[i];
		string vid = videofiles.ptr[i];

		const string mkvmerge_cmd_str = "mkvmerge -o %s/%s %s --language 0:%s --track-name 0:%s";
		const int mkvmerge_cmd_strlen = 1 + strlen(mkvmerge_cmd_str) + strlen(output_dir) + 2*strlen(vid) + strlen(lang) + strlen(language);
		string cmd = malloc(mkvmerge_cmd_strlen);
		int ret = sprintf(cmd, mkvmerge_cmd_str, output_dir, vid, vid, lang, language);
		if (ret < 0) {
			puts(program_error_msg(SPRINTF_ERR));
			return SPRINTF_ERR;
		}
		
		ret = pthread_create(&thread_id, NULL, run_cmd, (void*)cmd);
		if (ret != 0) {
			puts(program_error_msg(THREAD_CREATE_ERR));
			return THREAD_CREATE_ERR;
		}
		
	}
	ret = pthread_join(thread_id,NULL);
	if (ret != 0) {
		puts(program_error_msg(THREAD_JOIN_ERR));
		return THREAD_JOIN_ERR;
	}
	return 0;
}



int main(int argc, string argv[]) { 
	if (argc != 5) {
		puts(program_error_msg(ARG_COUNT_ERR));
		return ARG_COUNT_ERR;
	}
	int ret = addsubs(argv[1], argv[2], argv[3], argv[4]);
	if (ret != 0) {
		return ret;
	}

	return 0;
}
