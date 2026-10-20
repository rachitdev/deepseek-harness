# Chapter 12 — Physical Security Architecture

## Core Idea
The physical layer is **the builder's view — "the bricks and mortar of your enterprise security architecture."** Where the logical layer looked at functionality and flow, the physical layer looks at *"physical boxes, how many of them, where they are located, their size and performance and how much bandwidth you need to connect them together,"* at the **physical data structures** realising logical information structures, and at the **physical security mechanisms** implementing the logical security services.

> "A security mechanism is a physical means by which a logical security service is implemented."

The chapter fills the physical row: Business Data Model (What), Security Rules, Practices and Procedures (Why), Security Mechanisms (How), Users, Applications and the User Interface (Who), Platform and Network Infrastructure (Where), Control Structure Execution (When).

## Frameworks Introduced

### Rules vs practices vs procedures vs guidelines
| Artifact | Definition | Key property |
|---|---|---|
| **Rule** | *"A specific filter against which automated decisions are made by security sub-systems"* — firewall rules, database rules, file system rules. Often built into **ACLs**, which are made up of **access control entries (ACEs)**, each containing one or more rules | **Absolute and unequivocal.** "Whilst policies may require interpretation, rules do not" |
| **Practice** | Generic description of how to accomplish certain objectives in security management; the term is usually part of "good practice" or "best practice" — an approach found to work well elsewhere | More specific than policy, with definite behavioural content; less specific than a procedure |
| **Procedure** | Documented **step-by-step instructions** on how specific tasks are performed | **Product- and platform-specific.** "A new release of a given product may require the procedure to be re-written." A backup procedure for one platform will not apply to another — but the *practices* of data backup are the same for both |
| **Guideline** | Non-specific, like practices, but *"in the style of giving good advice to people about how to behave in order to comply with policy and in order to exhibit good practice"* | **Requires intelligent interpretation** |
| **Implementation guideline** | Covers the case where, given a specific policy and a supporting standard, it is still not clear how to build a specific piece of infrastructure | Advice on how to achieve it; less specific than a procedure |

### Database integrity control — the ACID-style properties the DBMS delivers
The functional description of the integrity control achieved by record locking:
- **Atomicity of transactions** — if the transaction terminates normally the alterations are made permanent (committed); if not, no alteration is made; the transaction is atomic (indivisible) and cannot be split.
- **Recoverability of transactions** — the recovery sub-system behaves such that the database contains **all** the effects of committed transactions and **none** of the effects of uncommitted transactions.
- **Serialisability of transactions** — each transaction accesses shared data without interfering with others; when two or more execute simultaneously the operations are interleaved; the interleaving is such that one transaction must complete before the next can begin.

**Four DBMS sub-systems** deliver this: **transaction manager** (pre-processes data and transactions), **scheduler** (controls ordering and serialisation), **recovery manager** (manages commitment and abortion), **cache manager** (acts directly on stored data).

**Database recovery mechanisms:** backups to create checkpoints — usually **full backups weekly, incremental daily**; **'before' image journaling**; **'after' image journaling**; **roll-back** (run the before-image journal in *reverse* chronological order against the checkpointed database); **roll-forward** (run the after-image journal in chronological order).

The **data structure hierarchy** (Figure 12-1): database → tables → records → fields. A sales database contains a customer table, orders table and products table; the customer table contains one record per customer; the customer record has fields such as customer number, name, address, credit limit. Locking granularity: a **file** system locks the whole file; a database locks at **record level**, because *"it would be unthinkable to lock the database so that only one user at a time could make use of it."*

### The four cryptographic services — and the rule for when not to use cryptography
| Service | What it does |
|---|---|
| **Confidentiality** | Preventing unauthorised disclosure of information |
| **Integrity** | Protecting content from alteration **without this being detected** — "you cannot prevent the alterations by using cryptography but you can ensure they will be detected" |
| **Authenticity** | Proving that information originated from an authentic trusted source |
| **Non-repudiation** | Preventing a dishonest party from later denying the authenticity of information provided by that party |

> **The decision rule:** *"if the security service you want to provide cannot be in some way presented as a variation of one of these four, then cryptography is not the right mechanism for your solution."*

Strength is *"mostly related to the number of bits in the cryptographic key and hence the number of possible key values"* — assuming no specific analytical weaknesses in the algorithm, so the cryptanalyst must search the whole key space.

### Entropy and the passphrase calculation
Key strength and password/passphrase strength are measured the same way — **entropy**, the number of truly random bits in a unit of data as opposed to redundant bits. **Human-readable text has high redundancy and hence low entropy density.** ASCII-encoded English text has **approximately 1.3 bits of real entropy per character**, so of the 8 bits per character, **6.7 are redundant**.

> "A common and naive mistake is to assume that a 16-character ASCII-encoded string can provide 128 bits of cryptographic key."

**The calculation:** for an English-language passphrase carrying 128 bits of entropy, you need **128 / 1.3 ≈ 99 characters**. This gives the user a double problem — remember a 99-character string (eased by making it a natural sentence, hence "passphrase") and type it with perfect accuracy. *"Nevertheless this approach provides a workable solution with high levels of key security."*

**The corollary nobody plans for:** if the master key is derived from an unstored passphrase and the user forgets it, there is no other way to recover the data keys. *"So, if you lose the passphrase, you lose the data — period! This means that for data recovery purposes the passphrase MUST be stored in written form in a physically secure location such as a safe."*

### The key management hierarchy
A **master key** is a key-encrypting key used to encrypt a population of working/data keys so those can be stored securely in an ordinary file. When a data key is needed it is retrieved and decrypted under the master key. **Only the master key then needs physically secure storage** — a tamper-resistant hardware device, or derivation from an unstored passphrase.

**Principles of key management (the book's list):**
- However strong the algorithms, if keys are poorly managed and can be learned by an opponent, the overall system is weak.
- *"In general it is a bad idea to allow human beings to have knowledge of cryptographic keys."* Humans are the weakest link; even the most honest have a price. *"That price may not be money… If you point a gun at someone's head the price becomes their own life. It probably gets worse if you point a gun at the heads of their children. It is actually very unfair to place employees in a position where they can be subjected to this type of duress."*
- Where human knowledge is unavoidable (emergency manual recovery), **split the key knowledge amongst two or more individuals**, each entering their own key component, so no single individual can be subjected to duress.
- Key management should otherwise be **completely automated**, using key-encrypting keys to move working data keys.
- Use a **truly random number generator**. *"Software random number generators cannot fulfil this requirement, because they always use some seed value that might be predictable, and they always use some mathematical function that is eventually cyclical."*
- **Tamper-resistant devices** can protect keys from disclosure.
- **Limit exposure to cryptanalysis**, defined by both the number of ciphertext samples an opponent can collect and the time available to work on them — the two trade off. A bulk data key generates large quantities of ciphertext and **should be changed frequently**; a key-encrypting key generates small samples and **can have its lifetime extended**. *"These principles are fundamental to the construction of key management hierarchies."*
- Use **different keys for different functions, different parties and different channels**, to ensure cryptographic separation of logical communications sharing the same physical bandwidth.

**The key management lifecycle — twelve stages:** generation; communication and distribution; storage; entry and installation; checking validity; usage; changing the active key; archiving; destruction; audit of key operations and usage; key backup and recovery; emergency reserve keys.

### In-line vs on-line cryptographic services architecture
| | **In-line** (Figure 12-10) | **On-line** (Figure 12-11) |
|---|---|---|
| Where | Networking layers of the system architecture | Applications layer |
| Services | OSI L1 confidentiality (encrypt the entire bit stream with in-line pairs of link encryption units); OSI L2 confidentiality (encrypt payloads in HDLC frames or Ethernet packets, in the NIC); OSI L3 confidentiality and integrity (encryption and checksums over X.25 or IP packet payloads — **IPSec transport mode**; **IPSec tunnel mode also gives some traffic flow confidentiality**); OSI L4 session authentication and transport payload confidentiality — **SSL**, via additional software in the TCP stack | Confidentiality of application payloads (whole or selected fields); integrity checking (whole or selected fields — **the selected-field option is especially useful where some field values vary over the data structure's lifetime and so cannot be in the checksum**); source authentication; peer-to-peer application-level authentication; non-repudiation; encryption of application data stored in files or databases; integrity checking of stored application data; user and entity authentication (the CAM architecture) |
| Key management | Most likely **in-line peer-to-peer**, though L3 systems may have a key distribution server | Can be **highly sophisticated** — many keys in the data store, encrypted under a master key held in a physically secure **tamper-resistant hardware unit** that may also offer accelerated cryptographic processing and a **hardware random number generator (RNG)** — a **hardware security module (HSM)** |
| Verdict | — | *"It offers by far the most flexible approach to providing cryptographic services to applications"* |

Typical on-line applications: e-mail and store-and-forward messaging; interactive messaging; file transfer; financial payment and money transfer systems in banking; point-of-sale and cash machine authorisation based on bank cards with PINs; secure eCommerce and eProcurement.

### Authentication exchange mechanisms — one-, two- and three-way
*(Footnote: "one-way", "two-way" and "three-way" refer to the number of **exchanges**, not the number of parties — in all three cases there are two parties, Alice and Bob. X.509 is the definitive reference.)*

| Mechanism | Structure | Notation |
|---|---|---|
| **One-way** (Fig 12-7) | Alice proves to Bob; Bob does not prove to Alice. Relies on a **pre-existing security association** sharing at least one secret key, whose creation may or may not involve a trusted key-distribution server | `E_AB{ SK, T, B }` where `E_AB{…}` = encrypted under a key previously agreed between Alice and Bob for Alice's use; `SK` = a random session key; `T` = a time stamp; `B` = Bob's name |
| **Two-way** (Fig 12-8) | The one-way protocol **reflected back**, so Bob also authenticates to Alice. Now you can have **a different session key for each direction** | `E_XY{ SK_XY, T_X, Y }` — `SK_XY` is the session key for the X→Y direction, `T_X` a time stamp made by X |
| **Three-way** (Fig 12-9) | **Replaces time stamps with random numbers** — a challenge-response protocol. Alice sends a random-number challenge (Alice is now the *verifier*, Bob the *prover*); Bob must decrypt, extract the nonce and return it re-encrypted under his own authentication key, sending his own challenge at the same time; Alice must do likewise | `R_X` = a random number made by X |

**Why each field is there:** the session key can be used for the rest of the session to encrypt data or provide integrity checksums (exchange *two* session keys if you want both). **The time stamp gives the exchange a limited life and prevents replay** — but Alice and Bob must agree on the time and need a trusted time service: *"If Bob's clock can be manipulated, then an opponent can replay out-of-time messages and masquerade as Alice to Bob."* **The inclusion of Bob's name prevents an opponent using the message to masquerade as Alice to any of Alice's other friends.**

### The seal / MAC / MIC mechanism (Figure 12-5)
Process the entire data contents through a cryptographic transformation producing a short checksum. If the algorithm needs **no key**, the method is **hashing**; there is also a keyed variant of the MAC technique, **hash-MAC or HMAC (keyed hashing)**. Hash values are usually used *inside* digital signature functions to improve efficiency and speed.
Flow: **the data structure itself is not encrypted and is still visible as plaintext.** The sender creates a seal and attaches it as an additional field. The receiver strips the received seal aside, generates a new seal with the same key, and compares. If they match, there is high assurance the data was not altered between the two seal-generating events.

### Digital signature mechanism (Figure 12-6) — why two stages
Applying an asymmetric algorithm directly to the target data would be very inefficient, slow, produce a signature the same size as the original data, **and make the cryptanalyst's job easier**. So: hash the data to a short hash value → digitally sign the hash → attach it. The verifier **rehashes the message**, recovers the original hash from the received signature, and compares. Private key signs; certified public key verifies. Digital signatures prove **both** the integrity of the data content and the authenticity of the source, and are **non-repudiable**.

### Public key certificates — why they exist
Public keys are known to all and are not confidential, **but they must be authentic**. *"An opponent could steal your identity and publish his own public key pretending to be you, unless there is some mechanism to prevent this."* The mechanism is **certification**: every public key in the community is digitally signed by a trusted **certification authority (CA)**, creating a **public key certificate**, and it is the certificate that is published. *"To be sure that you are using an authentic public key, you must first check the signature of the certification authority in the certificate."*

### Resilience — six principles and their application
**Principles:** avoidance of single points of failure (always an alternative mechanism for a given function or service); redundancy of hard physical components; backup and restoration procedures for all soft physical components (software and data); recovery procedures worked out in advance for all foreseeable failure scenarios; automated recovery and reconfiguration where possible; extensive event logging, monitoring and reporting **to help foresee possible failures before they occur** and to help recovery by providing evidence of the nature of the failure.

**Applied to network topology:** multiple communications cables and channels with **diverse physical routing**; separation of cable routes from buildings *"to avoid all the multiple cables suffering the same physical failure such as when the road is dug up"*; alternative telephone exchanges; alternative telco carriers where local market conditions allow; dynamic automated re-routing and reconfiguration for a **self-healing network** based on multi-path switches and connections; **ISDN or dial-up fallback** for leased lines; regular testing and monitoring of the resiliency features themselves; **duplicate frames and frame-rooms** in buildings where external telecom lines terminate; physical and environmental security of communications and computer rooms.

**Applied to host platforms:** dual processing facilities in separate data centres, **often geographically separated by several hundred kilometres**; fault-tolerant computer systems with special operating systems or middleware automatically organising data mirroring or distributed processing; RAID.

## Reference Tables

### Table 12-1 (extract): Mapping Security Mechanisms to Security Services
Every logical service from Chapter 11 gets one or more physical mechanisms. Representative rows:
| Logical security service | Physical security mechanisms |
|---|---|
| **Entity unique naming** | Naming standards; naming procedure; directory system |
| **Entity registration** | Registration policy; registration authority system; registration procedure |
| **Entity public key certification** *(and credentials certification — same list)* | Certification policy; certification authority system; certification procedure; certificate syntax standards; certificate publishing mechanism (directory); **certificate revocation list (CRL)**; CRL publishing and management (directory) |
| **Directory service** | Directory system; directory access protocols; directory object and attribute syntax rules; directory replication |
| **Entity authentication** | Login procedure; user passwords and tokens; client user agents for authentication; authentication exchange protocols; authentication server system; directory system |
| **Session authentication** | Mutual two-way and three-way authentication exchanges; **session context (finite state machine)** |
| **Message origin authentication** | Message source identifiers, protected by message integrity checksums, digital signatures, hashing |
| **Message replay protection** | Message nonce values protected by message integrity checksums |
| **Message contents confidentiality** | Message contents encryption; encryption key management; **routing control to physically secure networks** |
| **Non-repudiation** | Digital signatures; notarisation servers; transaction logs; trusted third party certification and arbitration |
| **Traffic flow confidentiality** | **Traffic padding** |
| **Authorisation** | Roles; fixed role associations with entities; real-time role association with entities; authorisation certificates |
| **Logical access control** | Local access control agents; local role ACLs; **central access manager (CAM)**; CAM role ACLs; central application access control agents; central application role ACLs; DBMS mechanisms; file system mechanisms |
| **Audit trails** | Event logs; event log integrity protection mechanisms; event log browsing tools; event log analysis tools; reporting tools |
| **Stored data confidentiality** | Logical access control; physical access control; stored data encryption; media storage security; **media disposal procedures** |
| **Software integrity protection** | Development lifecycle controls; delivery and installation controls; production system configuration control; production system change control; production system management authorisation; crypto-checksums on object code images; regular inspection of object code images and checksums; anti-virus tools |
| **Software licensing protection** | Software metering |
| **Trusted time** | Secure time server with clock; secure time server protocols |
| **User interface for security** | GUI login screens; GUI security message screens; single sign-on mechanism; **ergonomic design of authentication devices**; help desk for security problem resolution |
| **Security service management / provisioning / administration** *(identical mechanism sets)* | Security service management sub-system; secure management protocols; management agents in managed components; access control at all agents and sub-systems security alarms |
| **Security measurements and metrics** | Cryptographic test mechanisms; inspection tools; penetration testing; statistical tests |
| **Intrusion detection** | Intrusion signature analysis on network traffic; real-time system monitoring; alarms |
| **User support** | Help desk; trouble ticketing system |
| **Crisis management** | **Vested authority in a crisis manager and crisis management team**; assessment procedures; escalation procedures; activation procedures |
| **System audit** | Independent inspection; regular scanning with system audit tools |
| **Physical security** | Secure premises with locks and guards; locked rooms for servers, operations and communications; **physical protection for cabling**; authorisation procedures; identification badges and visitor procedures; supervision of contract engineers |
| **Environmental security** | Site-selection procedures; fire prevention, detection and quenching; flood avoidance, detection and removal; air temperature and humidity controls; electrical power protection mechanisms |

### Table 12-2: Comparison of Cryptographic Strengths of Algorithms
| Strength | Symmetric (key length) | Asymmetric (modulus length) | Hashing / MACs (hash value length) |
|---|---|---|---|
| **Weak** | DES 40-bit; RC4 40-bit | RSA 256-bit | — |
| **Medium** | DES 56-bit; CAST 64-bit | RSA 512-bit; Diffie-Hellman 512-bit; DSA 512-bit | ANSI X9.9 MAC 32-bit |
| **Strong** | Triple DES 112-bit; IDEA 128-bit; RC4 128-bit | RSA 1024-bit; Diffie-Hellman 1024-bit; DSA 1024-bit | MD5 128-bit; SHA-1 160-bit |
| **Very strong** | AES 192-bit; AES 256-bit | RSA 2048-bit; ECC 300-bit | SHA-256; SHA-384; SHA-512 |

**But algorithm strength is not the only indicator.** The other two major constraints: **the key management scheme** — does it prevent the keys from being revealed, and how are they managed? — and **the cryptographic protocols** — are they robust against replay attacks, man-in-the-middle attacks, spoofing attacks, dictionary attacks? *"To make a proper cryptographic assessment of a secure system is a job for an expert."*

*(Note: this table dates from 2005. The relative-strength method still stands; the specific thresholds — DES, MD5, SHA-1 — are long superseded. See Scope & Limits in SKILL.md.)*

### RAID levels as the book presents them
| Level | Description |
|---|---|
| **RAID 0** | Data striping across all physical disks, **without any parity information** to reconstruct damaged data. "There is no true redundancy in this solution, and hence its inclusion in the RAID definition is marginal" |
| **RAID 1** | Simple disk mirroring — data written simultaneously to two or more drives |
| **RAID 2** | Striping with additional parity bits (**Hamming codes**) for error detection and correction |
| **RAID 3** | Striping **byte-by-byte** to maximise distribution across the disk surfaces, with **parity bytes on a separate physical disk**. Any single failed drive can be completely reconstructed from the remaining drives |
| **RAID 4** | Like RAID 3 but striping on a **block** basis; different performance and cost profile — best choice depends on the read/write activity profile |
| **RAID 5** | Like RAID 4 but with **parity distributed amongst the drives**; different performance profile, same cost profile. Especially suited to **multi-user environments with many short, random write operations** |

### SQL security mechanisms
| Mechanism | Detail |
|---|---|
| **Three classes of user** | **System administrator** (access to and control of all databases in the entire DBMS); **database administrator** (a given database); **user** (privileges as defined by either administrator) |
| **GRANT / REVOKE** | `GRANT action_list ON object_name TO user_list` — `action_list` ∈ {SELECT, INSERT, UPDATE, DELETE, EXECUTE}; `object_name` ∈ {table_name, view_name, procedure_name}; `user_list` ∈ {user_name, group_name, public} |
| **Integrity constructs** | `UNIQUE` columns to prevent incorrect duplication; views created `WITH CHECK` to force underlying table updates to conform to value-range constraints; `COMMIT` and `ROLLBACK`; BEFORE IMAGE and AFTER IMAGE journaling with CHECKPOINTS |
| **Views** | Restricted access to a table — the user gets only the view, a sub-table containing certain rows and/or certain columns; a view can be defined **across multiple tables**; views may overlap arbitrarily |
| **Stored procedures** | Macros of SQL commands executed as a whole — *"you can grant quite sensitive operations to a user but only in the context of the macro and thus under close control"* |
| **Triggers** | Four types: **BEFORE statement**, **AFTER statement**, **BEFORE row**, **AFTER row**. Statement triggers fire **once per statement** regardless of rows processed; row triggers fire **on every row** |
| **Privilege hierarchy** | Privileges are **not checked in advance** and may conflict at execution time, so a hierarchy decides: **Procedure > Trigger > View > Table**. (Procedures and triggers use tables and views; views use tables) |
| **Audit logging** | Extensive by default, written to special system tables; extend and customise it **using triggers** to create your own audit log tables |
| **Roles** | Assign a user ID to a role (e.g. `PAYROLL-CLERK`) and control it in the application. The application must **pass across an identifier for the specific human user** so it can be written to a customised audit trail table for individual accountability; the DBMS must be configured to allow **multiple concurrent sessions under the single role user ID** |
| **Resource limits (profiles)** | Grouped into **profiles** assigned to one or more users: CPU time; number of concurrent sessions; idle time per session; input and output in logical data blocks |
| **Mandatory access control** | Implemented by some vendors for trusted-systems criteria (TCSEC/Orange Book B1, ITSEC F-B1/E3): each object (row) labelled with a compulsory security classification in an extra column, subjects assigned clearance levels, the DBMS matching the two |

### Two worked security triggers
- **Time limits for updates 08:30 to 18:00 Monday to Friday** — a **BEFORE statement** trigger checking the time.
- **Preventing employees from accessing their personal customer account details when they are both a customer and an employee** — a **BEFORE row** trigger checking the identity of the user.

### Two-phase commitment for distributed databases
Each physical location has its own **resource manager** controlling local resources and maintaining its own recovery log. One resource manager acts as **co-ordinator** (usually the one that initiates the transaction).
1. All participating resource managers prepare by **writing the effects of the transaction into non-volatile storage in their logs**. At this stage each can still COMMIT or ROLLBACK.
2. The co-ordinator awaits a message from every participant that it has reached this stage.
3. The co-ordinator writes an entry in its own log recording this status.
4. All participants are instructed to COMMIT.
5. When all have signalled completion, the transaction is completed.
6. **If any resource manager fails to complete normally, or there is a timeout because one is unable to respond, the co-ordinator issues ROLLBACK.**
7. **Residual risk:** a resource manager that has already committed may go off-line and be unable to roll back. This is detected by the co-ordinator and **a manual recovery operation is instigated**. The same approach is needed if the co-ordinator itself fails before overall completion.

### Password management — the book's position
| Guidance | Notes |
|---|---|
| Individual, not shared, passwords | Otherwise there is no accountability — *"not only is it impossible to identify who might have misused their privileges, but innocent users in the group then fall under suspicion"* |
| Non-disclosure; don't write it down where others can find it; don't lend your account | The standard good-practice advice |
| No easily guessed passwords; **never identical to the username** | — |
| Longer = more entropy = harder brute-force dictionary attack | Non-alphabetic characters raise entropy further |
| **But** with a highly parallel cracking engine most passwords can be cracked in finite time | "They provide only a finite level of security" — cross-reference to time-based security in Chapter 9 |
| **The 30-day change cycle is challenged** | *"Enforcing password changes on a regular basis (usually around every 30 days)… is even embodied in ISO 17799. However, as far as the authors are aware there is no scientific research that underpins this practice. It seems to be based purely on anecdotal wisdom. Why 30 days? This is probably not good practice at all"* — because many passwords with different syntax rules, different usernames and different change cycles guarantee confusion, and the user *"will certainly write down the passwords in a place of easy reference,"* breaking the non-disclosure rule |
| **The authors' common-sense alternative** | If you have a good password with strong syntax and you are sure it has not been disclosed, **keep it alive for several months — perhaps a year** |
| Change immediately if you believe it has been disclosed | — |
| **Always change vendor default passwords** | "The abuse of default passwords is a common vulnerability" |

Supporting reference the book cites: the **US DoD Password Management Guidelines CSC-STD-002-85 ("Green Book"), Appendix C, 1985** — provided the password file is not exposed to dictionary attack and attacks come only through the login interface, **there is no significant difference in vulnerability between a six-month and a twelve-month password life**; the **optimum password length is eight or nine random characters** (depending on alphabet size) and the **optimum passphrase is three words chosen randomly from a dictionary of 23,300 words.** Also cited: Tony Sale at COSAC 99, arguing that length and syntax rules actually *reduce* the number of possible permutations, and that capture of the German security procedure book — mandating frequency of change and order of cogs — dramatically reduced the power of Enigma.

### Platform (operating system) security — DAC mechanism list
Each real user has a **user account**; a **unique username and identifier** (UID in UNIX, SID in Windows); **groups** with group names and identifiers; a **password per account** with aging, minimum length and lock-out after successive failed logins; a **super-user account** (root / administrator) with access to everything; mapping of physical devices (disks, printers) onto logical file structures; **files and directories owned by their creator**, who grants access at his discretion; differentiated privileges (read data, write data, create file, delete file, execute program, modify privileges); a **home directory** per user; a **login program or script** controlling the view of the system granted; a **security kernel** enforcing process security — each process has a **process control block** in main memory holding the **UID of the owner**, whose privileges control the process's actions; an ability for some programs to **set the UID to a higher privilege** than the person running it (UNIX **SUID** — *"both a security-strength and a security-weakness, depending upon how it is used"*); event logs at various levels; **one-way encrypted password files**; password files stored in a super-user-privileged directory.

**Four major OS security issues:** the super-user is all-powerful — *"if an opponent can gain super-user privilege there is no limit to the damage he can inflict"*; carelessly coded privilege-elevating programs, where an opponent who elevates then breaks out of the program's constraints has gained great advantage; **Trojan horse attacks** — a Trojan version of a highly privileged system utility run with super-user privilege gives the opponent control (the book cites *The Cuckoo's Egg*, Clifford Stoll); **writeable directories**, which allow files to be added, renamed or removed *"whatever the permissions on the file itself."*

### Good-practice guidelines for a production platform
A written security policy supported by detailed standards aimed at those with operational responsibility; **production platforms segregated from development and test platforms**; file and directory permissions designed as part of the application; stringent change control on all executables; **regular scanning to detect changes in authorised executables and unauthorised installation of others**; strict control over hardware and software maintenance with close independent supervision; **all non-production executable software removed — especially no editors, compilers, linkers or development tools** (loaded temporarily under strict supervision for emergency maintenance and completely removed afterwards); a strict process for accepting and releasing new versions of production software; all default system accounts removed or given strong secret passwords; and **the super-user account must not be used for routine operations.**

### The super-user protocol (the book's "highly controversial" recommendation)
1. **All routine operations tasks fully analysed** to define what accesses are required to do the job.
2. **Captive scripts or programs** constructed that run with super-user privilege where needed, **without granting full super-user status to the person running the utility**.
3. **Every single** routine operations task subjected to this treatment.
4. The platform is then managed day-to-day **without the super-user account ever being required**.
5. The super-user password is **written down and locked in a safe under the control of a senior manager who has no hands-on computer operations duties**.
6. Only that senior manager decides to release it in an emergency.
7. **If "emergencies" are happening routinely, the analysis of routine operations tasks must be reworked** and new captive utilities added.
8. Once the super-user account has been used, **the password must be changed** and the new one stored in the safe.
9. **Password setting is the responsibility of a security administration team with no responsibility for computer operations.**

> "Any production shop that runs routinely with computer operators sharing the super-user password to operate the platforms by issuing raw command-line syntax is a disaster waiting to happen. However, this can be seen in practice in a number of computer centres!"

### Hardware security — the two traditional approaches and their limits
| Approach | Limit |
|---|---|
| **Physical enclosure** — the whole platform and peripherals in a secure computer centre / data centre, with controlled access and rigorous software release procedures | *"Inappropriate when the platform to be secured is a personal computing device — a desktop or laptop PC or a PDA"* |
| **Tamper-resistant modules (TRMs)** — used especially for secure storage and processing of cryptographic keys, both secret and private. Levels specified in **FIPS PUB 140-1 / 140-2** and beyond by specialist vendors | The **HSM architecture** (Figure 12-12) still requires the application software and host platform to be inside a physically secure suite: *"whatever authorisation decision is taken inside the secure TRM, once that decision is exported back to the application, it can be subverted. The TRM is only a slave to the application and has no control over the security of the application itself."* Smart cards on PCs are **architecturally the same as HSMs on mainframes** and share the same limitation |

*(Terminology note the book insists on: "Some people talk about 'tamper-proof' equipment, but this is a strong claim indeed… 'Tamper-resistant' is a much more realistic term.")*

**The forward-looking answer: chip-level security.** The **Trusted Computing Group (TCG)** promotes open standards for trusted computing hardware building blocks; Intel's announced **LaGrande Architecture** targets secure *client* platforms — any platform users have direct access to and which can initiate communications with other platforms. It counters malicious software that can **read memory and expose secrets; change memory and alter data or code functionality; manipulate input or output; change requests for information from other devices.**
**LaGrande's four design principles:** protected execution of trusted code; **attestation** — proving the trustworthiness of the hardware platform and its current configuration; sealed storage of data and code; **trusted communication channels and paths** for input and output through keyboards, graphics displays and other peripherals.
**The unsolved problem:** a trusted path requires the user's cooperation — the user must be *informed* the path is in place and modify behaviour accordingly. How? A screen symbol like the SSL padlock, or an illuminated LED on the peripheral? *"Can this indicator be spoofed by unauthorised software? Of course it can! Hmmm."*

### Control structure execution mechanisms (the physical When cell)
- **Date and time fields embedded in renewable data structures** such as certificates — the certificate cannot be used past its expiry because every recipient checks the date as a routine control point in the acceptance process.
- **Date and time fields in configuration files** telling you when a control event is due, with a regular lookup comparing current date/time against the event threshold — e.g. the next virus-scanner update. *(Footnoted expert opinion from Niels Bjergstrom, 14 May 2003: every one or two days is the appropriate update threshold.)*
- **Automated timers** started at the opening of a maximum-lifetime period — a login session lifetime of several hours, an inactivity timeout of several minutes, a protocol timeout of several seconds — generating an interrupt on expiry that triggers a control point and terminates the wait state.

## Worked Example

### Figure 12-13: The physical network topology for the Chapter 11 logical domain model
Same organisation, same requirements — now realised in *"real boxes — routers, firewalls, server platforms, client platforms, etc., and real network connections, both LAN and WAN."* The mapping between the two diagrams is the point of the exercise: *"Once you have accurately captured the logical domains it is relatively straightforward to design the physical implementation of them. However, it is difficult to miss out the logical domain step and still get the physical topology correct."*

The eight main features:
1. **The production domain is contained behind internal firewalls in the data centre.** The firewall architecture comprises a combination of **dual-homed bastion hosts and screening routers**.
2. **Public Internet access to web services goes via a screening router** that leads only to the external LAN carrying the web server. That **screened sub-net is then double-buffered from the production domain by two bastion hosts separated by a DMZ** — a demilitarised zone on which no services are deployed. Double buffering is adopted *"because of the high level of threat from the public Internet."*
3. **Internal firewalls on all corporate WAN connections** isolate the production domain from all other domains.
4. **The VPN host** — another dual-homed bastion host — gives a **second double-buffered route to the Internet** and carries traffic from home workers, road warriors and business partners. This traffic is the physical realisation of the logical extranet services.
5. **VPN clients** for home workers and road warriors physically implement the logical extranet.
6. **A VPN host at each business partner.** The ownership boundary matters: *"this VPN host is part of the deployment of the main organisation and implements the main organisation VPN security policy, whereas the secondary bastion host shown in the business partner site is owned entirely by the business partner, enforcing their security policy and protecting them from attack by any party, including the main organisation."* The partner's other router connecting their DMZ directly to the Internet also belongs to them and handles no traffic to or from the main organisation.
7. **Partner services and external services servers sit on separate screened sub-nets** well away from the main production domain.
8. **No one has direct access to the main application servers except data centre operations staff.** All user access is via internal firewalls across the corporate WAN; **all extranet access is via intermediate servers — never directly to the applications themselves.** Partner services communications run at *application level* between the partner services server in the data centre and a remote partner services server installed at each business partner site.

### Figure 12-14: The directory physical topology
Design issues to be addressed: resilience; service availability; performance including response time; balance between enquiries and updates; replication strategy.

- **Two master directory copies with real-time mirroring** between them, so that if one goes off-line or is destroyed the other has a complete copy. *"An even greater degree of resilience could be achieved by having three separate master copies."*
- **Geographic separation** so the masters cannot suffer the same destructive event — different sites, hundreds of kilometres apart; for an international organisation, **different continents. "Master 1 might be in London and Master 2 in New York. For the triple-resilient model Master 3 might be in Singapore, Hong Kong or Sydney."**
- **Local slave replicas handle user enquiries.** Serving every enquiry from the masters would need prohibitively expensive bandwidth, risk overload if any bandwidth were lost, and give unacceptably long response times at peak. Local slaves cut long-distance traffic and cost, allow realistic capacity planning, and shorten the return route. **Replication engines** perform the replication.
- **The replication strategy trade-off.** Immediately after an update from a master, a replica is **consistent and convergent**; as further master updates accumulate un-replicated, replicas become **inconsistent and divergent**. One factor to weigh: *"the potential liability incurred by replica slave directories holding out-of-date credentials, such as revoked public key certificates."*
- **Dual local connections** — offer each user two slave replicas so one can cover the other, but capacity planning must allow for **peak loads when certain replicas are unavailable**.
- **The topology's implicit constraint:** updates can be made **only to one of the mirrored masters**; the slaves handle queries only. *"If you have a directory where frequent updates are required from widely distributed directory clients then you will need a different topology"* — likewise if divergence is a major business issue.
- **Local authorisation servers.** Because RBAC is built around a central access manager (CAM) that is similar to and closely associated with the directory, *"wherever there is a slave replica for the directory, there should also be a distributed slave authorisation server, which uses as its database the directory itself."*

### Vulnerability Case Study 1 — the encrypted password that could simply be replayed
An IBFS division authenticated users of a client-server application with a username and password sent across the internal LAN to the server. The application was then extended to branch offices connecting via **public dial-up**, raising the concern that passwords could be intercepted at the local loop or an intermediate exchange.

**The fix as designed:** an additional routine in the client software to **encrypt the password before sending it**; the server decrypts before checking. *"Simple! The designers even designed a reasonable but easy-to-implement key management scheme, the detail of which is irrelevant."*

**What is wrong:** *"in order to attack the system from the outside you do not need to decrypt the password. What the server expects to receive is an encrypted password. So the opponent can listen on the line, capture the encrypted password, and replay the same encrypted password at another time so as to masquerade as the authentic user."*

**The correct design:** the password block must have a **nonce** inserted — a value used only once, tracked by both client and server; typically time stamps, random numbers, sequence numbers, or a combination. The password block then becomes a one-time value that cannot be re-used.

**The lesson:** *"This is not a failure of the cryptographic algorithm — it is a failure of the cryptographic protocol… Cryptographic protocol design is a subtle business and should not be undertaken by amateurs. Using homegrown protocols as in this case study is a recipe for a disaster. You are strongly advised not to do it."*

### Vulnerability Case Study 2 — the box of listing paper
The IBFS data centre runs 365×24, but the night shift is a low-profile affair with **one single operator on duty for much of the time**. The only way into the computer suite is through one of a pair of **glass tubular doors that allow only one person at a time**. The authorised person presents a smart card at a reader, the outer door opens, they remove the card and step in; **a weight sensor on the floor** detects an occupant and closes the outer door; after a **15-second delay** the inner door opens. Exit is the same in reverse from an inside reader, so you must carry your card.

Rumour said a male night-shift operator was bringing his girlfriend in. The data centre manager thought this unlikely — *"even if the girl were a small person, it was simply infeasible for two people to get into the tube together."* The rumours persisted; she installed a hidden CCTV camera and viewed the tape.

**The attack:** the operator timed the girlfriend's arrival for after everyone had left, came out and let her into the building. At the computer suite door he entered normally. He then fetched **a heavy box of listing paper used on the high-speed printer** and put it into the tube — having inserted his card first, then withdrawn it and **placed it on top of the box**. *"The door operated as if a person were in there."* When the far side opened, the girlfriend took out the box, set it aside, and used the card to enter. To exit, the procedure was reversed: one went out, put the box back in with the card on top; the other removed the box, returned it, and exited normally.

**The lesson:** *"The door mechanism operated normally and remained strong. What failed was the protocol for using the door. Vulnerabilities are often subtle, and that is why designers do not see them. However strong and well-designed you think your security mechanisms are, you should never be overconfident, and you should always get other people to try to crack them as a test. **The vulnerabilities usually lie in the unplanned functionality or in the abuse of the protocol for using the mechanism.**"*

### The SeaView lesson on model-driven design (Dorothy Denning, quoted at length)
> "I learned my second lesson on the limits of models in the mid-80s while working at SRI. I was part of a project to develop a model for a multilevel-secure database system based on views. Our model, which we called SeaView, grew progressively more complex as we attempted to address the real issues. Any hope of usability had been killed by a concept called **polyinstantiation**, which involved instantiating multiple data values within a single field of a record, all with different security classifications.
>
> "Polyinstantiation was needed to satisfy the mathematical models of multilevel security, but it got uglier and uglier the deeper we went. **I learned then that security models could lead to dreadful systems that nobody would ever use.**"

## Mental Models
- **A mechanism implements a service.** One logical service, many candidate mechanisms; Table 12-1 is the join.
- **Cryptography does exactly four things.** If your requirement is not a variation of confidentiality, integrity, authenticity or non-repudiation, cryptography is the wrong answer.
- **Integrity mechanisms detect; they do not prevent.**
- **Entropy, not character count.** 1.3 bits per English character; a 99-character passphrase for a 128-bit key.
- **The protocol fails, not the algorithm.** Both vulnerability case studies are protocol failures around a mechanism that worked exactly as designed.
- **Key exposure = samples × time**, and the two trade off — hence bulk data keys change often and key-encrypting keys live long. That is the whole logic of the key hierarchy.
- **The TRM is a slave to the application.** Hardware security that cannot vouch for the software above it can always be subverted at the boundary.
- **Practices are portable; procedures are not.** Backup practices are the same on any platform; the backup procedure is not.

## Anti-patterns
- Encrypting a password for transmission without a nonce — the encrypted block is simply replayed.
- Home-grown cryptographic protocols.
- Assuming a 16-character ASCII string yields a 128-bit key.
- Storing the file-encryption keys in another file on the same system — *"then no advantage has been gained."*
- No physical written backup of a passphrase-derived master key: lose the passphrase, lose the data, period.
- Software random number generators for key generation — predictable seed, eventually cyclic function.
- Placing a single employee in a position where duress can extract the whole key — split key components instead.
- Reusing the same key across functions, parties and channels.
- Claiming equipment is "tamper-proof".
- A trusted-path indicator that unauthorised software can spoof.
- Routine operations run by operators sharing the super-user password and issuing raw command-line syntax.
- Leaving editors, compilers, linkers and development tools on a production platform.
- Leaving vendor default passwords in place.
- Enforcing a 30-day password change cycle across many systems with differing syntax rules and usernames — it guarantees written-down passwords.
- Writeable directories, and carelessly coded privilege-elevating (SUID-style) programs.
- Group-shared passwords, which destroy accountability and taint the innocent.
- Letting a security model drive the design until nobody will use the result (SeaView / polyinstantiation).
- Skipping the logical domain model and designing the physical topology directly.

## Key Takeaways
- Physical = builder's view: boxes, bandwidth, locations, data structures, mechanisms.
- File systems give **discretionary access control** plus file locking; DBMSs give record-level locking with atomicity, recoverability and serialisability, delivered by four sub-systems and journal-based recovery.
- SQL is itself a security toolkit: GRANT/REVOKE, views, stored procedures, four trigger types, a procedure > trigger > view > table privilege hierarchy, resource profiles and customisable audit logging.
- Rules are absolute; practices are generic and portable; procedures are product-specific; guidelines require interpretation.
- Table 12-1 maps every Chapter 11 logical service to concrete mechanisms — this is the physical layer's central deliverable.
- Cryptography provides exactly four services; strength depends on key length, but key management and protocol design matter at least as much.
- Encryption (symmetric/asymmetric/one-way), seals (MAC, MIC, hash, HMAC), public key certificates, digital signatures (two-stage: hash then sign), and one-/two-/three-way authentication exchanges are the core mechanism set.
- Key management has twelve lifecycle stages and a set of principles built around minimising human knowledge and limiting cryptanalytic exposure.
- Two cryptographic service architectures: in-line (network layers) and on-line (application layer, HSM-backed) — the on-line one is by far the more flexible.
- Every mechanism has undiscovered vulnerabilities; the two case studies show both are **protocol** failures, and the answer is professional penetration testing.
- Resilience is six principles applied to networks (diverse routing, alternative exchanges and carriers, self-healing, dial-up fallback) and platforms (dual data centres hundreds of kilometres apart, fault-tolerant systems, RAID).
- The super-user account should never be used for routine operations; captive privileged utilities plus a password in a safe under a non-operations senior manager is the recommended protocol.
- The physical When cell is implemented with expiry dates in renewable structures, date/time thresholds in configuration files, and automated timers generating interrupts.

## Connects To
- [Chapter 3 — Security Architecture Model](ch03-security-architecture-model.md) — the physical row of the SABSA Matrix.
- [Chapter 7 — Using This Book as a Practical Guide](ch07-using-this-book-as-a-practical-guide.md) — Figure 7-10, the physical layer deliverables including Physical Layout and Capacity Plan and Resilience Model.
- [Chapter 9 — Contextual Security Architecture](ch09-contextual-security-architecture.md) — time-based security, which bounds how long a password resists cracking.
- [Chapter 10 — Conceptual Security Architecture](ch10-conceptual-security-architecture.md) — security lifetimes and deadlines that these control-structure mechanisms enforce; VPN and firewall domain analysis.
- [Chapter 11 — Logical Security Architecture](ch11-logical-security-architecture.md) — the security services that Table 12-1 maps to mechanisms, and the logical network domain model that Figure 12-13 realises.
- [Chapter 13 — Component Security Architecture](ch13-component-security-architecture.md) — the products, tools and standards that instantiate these mechanisms.
- [Chapter 14 — Security Policy Management](ch14-security-policy-management.md) — concrete examples of the policy / standard / procedure / guideline distinction.
- [Chapter 16 — Assurance Management](ch16-assurance-management.md) — penetration testing as the way vulnerabilities in mechanisms are found.
- [Chapter 17 — Security Administration and Operations](ch17-security-administration-and-operations.md) — running the production platform under these good-practice rules.
