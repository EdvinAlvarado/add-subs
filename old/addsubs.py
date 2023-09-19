#!/usr/bin/env python

import sys
import os
import re

def addsubs(path: str, videoformat: str, subformat: str, lang: str) -> int:
    dir_list: list[str] = os.listdir(path)
    # print(dir_list)
    
    # ISO 639.2
    # https://www.loc.gov/standards/iso639-2/php/code_list.php
    langs: dict[str, str] = {"jpn": "Japanese", "eng": "English", "spa": "Spanish", "und":"Undetermined"}
    if lang not in langs:
        print("unsupported language")
        return 1

    re_vid: str = f".{videoformat}$"
    re_sub: str = f".{subformat}$"

    videos = [f for f in dir_list if re.search(re_vid, f)]
    subs = [f for f in dir_list if re.search(re_sub, f)]
    if len(videos) != len(subs):
        print("MISMATCH AMOUNT OF VIDEO FILES AND SUBTITLE FILES")
        return 2
    
    files: dict[str,str] = dict(zip(subs, videos))
    print("Joining subs files to these video files.") 
    for sub,vid in files.items():
        print(f"{sub}\t{vid}")

    answer = input("Are these pairs correct? (Y/n): ")
    if answer.find("n") != -1:
        return 3

    os.system("mkdir output")
    for s,v in files.items():
        o = '/'.join(["output", v])
        os.system(f"mkvmerge -o '{o}' '{v}' --language 0:{lang} --track-name 0:{langs[lang]} '{s}'")
    return 0

if __name__ == "__main__":
    sys.exit(addsubs(*sys.argv[1:]))
