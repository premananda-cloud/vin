.PHONY: paper clean
paper:
	cd latex && latexmk -pdf main.tex
	cp latex/main.pdf submission/main.pdf
clean:
	cd latex && latexmk -C
