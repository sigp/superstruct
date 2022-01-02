docs:
	rm -rf $@
	cargo doc --examples
	mdbook build book
	mkdir -p $@
	cp -r book/book/* $@
	cp -r target/doc $@/rustdoc

clean:
	rm -rf docs

.PHONY: docs clean
