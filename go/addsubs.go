package main

import (
	"fmt"
	"log"
	"os"
	"os/exec"
	"sort"
	"strings"
	"sync"
)

type ProgramError struct {
	Err string
}

const (
	MismatchError string = "Not the same amount of video and sub files"
	LangError            = "Language not supported"
	ExitError            = "User Cancelled"
	InputError           = "IO error"
)

func (pe *ProgramError) Error() string {
	return fmt.Sprintf("Program Error: %v", pe.Err)
}

type Output struct {
	output []byte
	err    error
}

func AddSubs(dir string, videoformat string, subformat string, lang string) ([]Output, error) {
	// ISO 639.2
	langs := map[string]string{
		"jpn": "Japanese",
		"eng": "English",
		"spa": "Spanish",
		"und": "Undetermined",
	}
	if _, ok := langs[lang]; !ok {
		return nil, &ProgramError{LangError}
	}

	// create list of files
	videofiles := make([]string, 0, 12)
	subfiles := make([]string, 0, 12)
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, err
	}
	for _, entry := range entries {
		if !entry.IsDir() {
			filename := entry.Name()
			if strings.Contains(filename, videoformat) {
				videofiles = append(videofiles, filename)
			} else if strings.Contains(filename, subformat) {
				subfiles = append(subfiles, filename)
			}
		}
	}

	// Check length
	if len(videofiles) != len(subfiles) {
		return nil, &ProgramError{MismatchError}
	}

	// Show user
	sort.Sort(sort.StringSlice(videofiles))
	sort.Sort(sort.StringSlice(subfiles))
	fmt.Println("Joining sub files to these video files")
	for i, sub := range subfiles {
		fmt.Printf("%s\t%s", sub, videofiles[i])
	}

	// User Confirmation
	fmt.Print("Are these pairs corret? (Y/n): ")
	var confirmation string
	fmt.Scanln(&confirmation)
	if strings.Contains(confirmation, "n") {
		return nil, &ProgramError{ExitError}
	}

	// Make output directory
	cmd := exec.Command("mkdir", "output")
	err = cmd.Run()
	if err != nil {
		return nil, err
	}

	// Run mkvmerge for all the files
	var wg sync.WaitGroup
	ch := make(chan Output, len(subfiles))
	for i, s := range subfiles {
		wg.Add(1)
		v := videofiles[i]
		defer fmt.Println(v)

		args := strings.Join([]string{
			"-o",
			dir + "/output/" + v,
			v,
			"--language",
			"0:" + lang,
			"--track-name",
			"0:" + langs[lang],
			s}, " ")

		go func(args string) {
			defer wg.Done()
			cmdMerge := exec.Command("mkvmerge", args)
			out, err := cmdMerge.Output()
			ch <- Output{out, err}
		}(args)
	}

	wg.Wait()
	close(ch)
	ret := make([]Output, 0, len(subfiles))
	for output := range ch {
		ret = append(ret, output)
	}

	return ret, nil
}

func main() {
	fmt.Println("vim-go")
	results, err := AddSubs(os.Args[1], os.Args[2], os.Args[3], os.Args[4])
	if err != nil {
		log.Fatal(err)
	}
	for _, output := range results {
		stdout := output.output
		err := output.err
		if err != nil {
			log.Fatal(err)
		}
		fmt.Println(string(stdout))
	}
}
