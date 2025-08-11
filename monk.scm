;;; GNU Guix package definition for Monk
;;; A simple Git hooks manager written in Rust
;;;
;;; To install:
;;; guix package -f monk.scm

(use-modules (guix packages)
             (guix download)
             (guix git-download)
             (guix build-system cargo)
             (guix licenses)
             (gnu packages pkg-config)
             (gnu packages crates-io)
             (gnu packages crates-apple)
             (gnu packages crates-windows))

(define-public monk
  (package
    (name "monk")
    (version "0.3.3")
    (source
     (origin
       (method git-fetch)
       (uri (git-reference
             (url "https://github.com/daynin/monk")
             (commit "main")))
       (file-name (git-file-name name version))
       (sha256
        (base32 "0s29q107haf0kmrl05zqqncgmmkj77hflhcyb6s6w19f0vfsmfii"))))
    (build-system cargo-build-system)
    (arguments 
     `(#:install-source? #f
       #:tests? #f  ; Skip tests to avoid dependency issues
       #:phases
       (modify-phases %standard-phases
         (add-after 'unpack 'fix-cargo-deps
           (lambda _
             ;; Adjust dependency versions to match what's available in Guix
             (substitute* "Cargo.toml"
               (("4.5.41") "4.5.23")
               (("1.0.219") "1.0.216")
               (("0.9.34") "0.9.30"))
             ;; Remove Cargo.lock to regenerate with new versions
             (delete-file "Cargo.lock")
             #t)))
       #:cargo-inputs
       (("rust-clap" ,rust-clap-4)
        ("rust-serde" ,rust-serde-1)
        ("rust-serde-yaml" ,rust-serde-yaml-0.9))
       #:cargo-development-inputs
       (("rust-clap-derive" ,rust-clap-derive-4)
        ("rust-serde-derive" ,rust-serde-derive-1))))
    (native-inputs
     (list pkg-config))
    (home-page "https://github.com/daynin/monk")
    (synopsis "Simple Git hooks manager written in Rust")
    (description
     "Monk is a simple Git hooks manager written in Rust.  It allows you to
manage and automate Git hooks easily using a YAML configuration file.  With
Monk, you can define hooks for various Git events (pre-commit, post-commit,
pre-push, etc.) and run custom scripts or commands automatically.")
    (license expat)))

;; Return the package for direct installation
monk
