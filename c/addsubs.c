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
	ARG_COUNT_ERR,
	THREAD_CREATE_ERR,
	THREAD_JOIN_ERR,
};

typedef struct	{
	string *ptr;
	size_t len;
	size_t size;
} StrVec;

void append_strvec(StrVec *v, string s) {
	v->len += 1;
	if (v->len > v->size) {
		v->size *= 2;
		v->ptr = realloc(v->ptr, v->size * sizeof(string));
	}
	strcpy(*(v->ptr + v->len), s);
}

StrVec new_strvec(size_t size) {
	StrVec ret;
	ret.size = size;
	ret.len = 0;
	ret.ptr = (string *)calloc(size, sizeof(string *)); // check if need to calloc internals;
	return ret;
}

void free_strvec(StrVec* v) {
	for (int i = 0; i <= v->len; i++) {
		free(v->ptr + i);
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
		return LANGUAGE_ERR;
	}

	StrVec videofiles = new_strvec(12);
	StrVec subfiles = new_strvec(12);

	DIR *folder = opendir(dir);
	if (folder == NULL) {
		puts("error: that is not a directory.");
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
		puts("error: either there are no video files or sub files that meet the file format specified.");
		return NO_FILE_ERR;
	}
	if (videofiles.len != subfiles.len) {
		puts("error: no equal ammount of video and sub files.");
		return MISMATCH_ERR;
	}

	qsort(&videofiles, videofiles.len, sizeof(string), compare_str);
	qsort(&subfiles, subfiles.len, sizeof(string), compare_str);

	puts("Are these pairs correct? (Y/n):");
	char response[10];
	if (scanf("%s", response) < 0) {
		puts("error: response was too long?");
		return RESPONSE_ERR;
	}
	if (strstr(response, "n") != NULL) {return USER_CANCEL;}

	char output_dir[100] = "";
	strcpy(output_dir, dir);
	strcat(output_dir, "/output");
	int ret = mkdir(output_dir, 0777);
	if (ret != 0 && errno != EEXIST) {
		puts("error: mkdir error out unexepectedly. Check permissios on the directory.");
		return OUTPUT_MKDIR_ERR;
	}

	// TODO test loop and join
	pthread_t thread_id;
	for (int i = 0; i < subfiles.len; i++) {
		string sub = subfiles.ptr[i];
		string vid = videofiles.ptr[i];

		char cmd[1000];
		int ret = snprintf(cmd, 1000, "mkvmerge -o %s/output/%s %s --language 0:%s --track-name 0:%s", dir, vid, vid, lang, language);
		if (ret < 0) {
			puts("error: encoding failed or command line string too long.");
			return SNPRINTF_ERR;
		}
		
		ret = pthread_create(&thread_id, NULL, run_cmd, (void*)cmd);
		if (ret != 0) {
			puts("error: thread create failed.");
			return THREAD_CREATE_ERR;
		}
		
	}
	ret = pthread_join(thread_id,NULL);
	if (ret != 0) {
		puts("error: pthread join failed.");
		return THREAD_JOIN_ERR;
	}
	return 0;
}



int main(int argc, string argv[]) { 
	if (argc != 5) {return ARG_COUNT_ERR;}
	int ret = addsubs(argv[1], argv[2], argv[3], argv[4]);
	if (ret != 0) {
		return ret;
	}

	return 0;
}
