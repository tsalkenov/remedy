# remedy
Simple cli program to play gifs directly into terminal representing each pixel as unicode symbol

### Usage
Simply call executable with your gif file as the first argument
```
$ remedy example.gif
```
You can change character representing one pixel you can specify it with --char flag
```
$ remedy example.gif --char "#"
```
If you are having issues with resolution of the frames
all debug information can be displayed with --debug flag
```
$ remedy example.gif --debug
 INFO  remedy > multiplier: 0.08835341
 INFO  remedy > term: 171x44
 INFO  remedy > img: 498x498
 INFO  remedy > output: 88x44
```
