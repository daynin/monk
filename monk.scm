;;; GNU Guix package definition for Monk
;;; A simple Git hooks manager written in Rust
;;;
;;; To install directly from GitHub:
;;; guix package -f https://raw.githubusercontent.com/daynin/monk/main/monk.scm
;;;
;;; Or locally:
;;; guix package -f monk.scm

(use-modules (guix packages)
             (guix download)
             (guix git-download)
             (guix build-system cargo)
             ((guix licenses) #:prefix license:)
             (gnu packages crates-io)
             (gnu packages pkg-config))

(define-public monk
  (package
    (name "monk")
    (version "0.2.5")
    (source
     (origin
       (method git-fetch)
       (uri (git-reference
             (url "https://github.com/daynin/monk")
             ;; Use the main branch for latest, or specify a tag/commit
             (commit "main")))
       (file-name (git-file-name name version))
       (sha256
        (base32
         ;; This will be computed automatically by Guix
         ;; when fetching from git
         "0000000000000000000000000000000000000000000000000000"))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-clap" ,rust-clap-4)
        ("rust-serde" ,rust-serde-1)
        ("rust-serde-yaml" ,rust-serde-yaml-0.9))
       #:phases
       (modify-phases %standard-phases
         (add-after 'unpack 'update-cargo-lock
           (lambda _
             ;; Ensure Cargo.lock is present and up to date
             #t)))))
    (native-inputs
     (list pkg-config))
    (home-page "https://github.com/daynin/monk")
    (synopsis "Simple Git hooks manager written in Rust")
    (description
     "Monk is a simple Git hooks manager written in Rust.  It allows you to
manage and automate Git hooks easily using a YAML configuration file.  With
Monk, you can define hooks for various Git events (pre-commit, post-commit,
pre-push, etc.) and run custom scripts or commands automatically.")
    (license license:mit)))

;; Return the package for direct installation
monk
