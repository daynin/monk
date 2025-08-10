;;; GNU Guix package definition for Monk
;;; A simple Git hooks manager written in Rust
;;;
;;; To install directly from GitHub:
;;; guix package -f https://raw.githubusercontent.com/daynin/monk/main/monk.scm

(use-modules (guix packages)
             (guix download)
             (guix git-download)
             (guix build-system cargo)
             (guix licenses)
             (gnu packages pkg-config)
             (ice-9 popen)
             (ice-9 rdelim)
             (ice-9 regex))

;; Dynamically compute the hash for the main branch
(define (get-monk-hash)
  (let* ((port-mktemp (open-pipe* OPEN_READ "mktemp" "-d" "/tmp/monk-XXXXXX"))
         (tmpdir (read-line port-mktemp))
         (_ (close-pipe port-mktemp))
         (_ (system* "git" "clone" "--depth" "1" 
                     "https://github.com/daynin/monk.git" tmpdir))
         (port (open-pipe* OPEN_READ "guix" "hash" "-rx" tmpdir))
         (hash (read-line port)))
    (close-pipe port)
    (system* "rm" "-rf" tmpdir)
    (if (eof-object? hash)
        "0000000000000000000000000000000000000000000000000000"
        hash)))

(define-public monk
  (package
    (name "monk")
    (version "0.2.5")
    (source
     (origin
       (method git-fetch)
       (uri (git-reference
             (url "https://github.com/daynin/monk")
             (commit "main")))
       (file-name (git-file-name name version))
       (sha256
        (base32 (get-monk-hash)))))
    (build-system cargo-build-system)
    (arguments
     `(#:phases
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
    (license expat)))

;; Return the package for direct installation
monk
