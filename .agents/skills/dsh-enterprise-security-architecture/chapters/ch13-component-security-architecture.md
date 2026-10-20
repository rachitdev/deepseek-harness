# Chapter 13 — Component Security Architecture

## Core Idea
The component layer is **the tradesman's view — "the specialised tools and product components of your enterprise security architecture."** The chapter deliberately *"stops short of discussing any specific brand or vendor"* for two reasons: *"(1) it is not appropriate to comment on one versus another here and (2) the marketplace changes so rapidly that the authors have focused on writing as timelessly as possible by sticking to generic principles."* At this layer *"there is little to say on most of the cells."*

The load-bearing idea is **inter-operability through standards**: *"If every component were built to a unique interface specification, nothing would ever work together."* And the wry counterpoint: *"Having agreed that a standard should make everything the same, you then find that there are so many standards from which you can choose!"*

The chapter fills the component row: Detailed Data Structures (What), Security Standards (Why), Security Products and Tools (How), Identities, Functions, Actions and ACLs (Who), Processes, Nodes, Addresses and Protocols (Where), Security Step Timing and Sequencing (When).

## Frameworks Introduced

### ASN.1 — Abstract Syntax Notation One (ISO/IEC 8824)
*"A strongly typed language especially designed for specifying application-layer protocol data units for communications protocols of all sorts."* In security it defines authentication exchange data structures, digital signatures, seals and digital certificates.

| Category | Types |
|---|---|
| **Simple built-in types** | BIT STRING, BOOLEAN, CHARACTER STRING, ENUMERATED, INTEGER, OCTET STRING, NULL, REAL |
| **Time types** | **UTCTime** — Co-ordinated Universal Time (GMT); **GeneralizedTime** — local date and time including a difference from GMT |
| **CHARACTER STRING classes** | GraphicString, IA5String (international alphabet number 5), NumericString, PrintableString, TeletexString, VideotexString, VisibleString |
| **Structured types** | SEQUENCE, SET, CHOICE — used to construct new types by combining simple types |
| **OBJECT IDENTIFIER** | An external reference to an ASN.1 object described and registered by an international authority; the reference is a series of integers that are **the node numbers on a hierarchical tree beginning at the root** |
| **Encoding rules** | **BER** (Basic Encoding Rules) or **DER** (Distinguished Encoding Rules) — DER is the 1993 improvement that **guarantees a unique encoding of every object**. Actual encoding is achieved through special compilers |

**Worked OBJECT IDENTIFIER (Figure 13-1)** — the MD5 hashing algorithm is `{1, 2, 840, 113549, 2, 5}`, read down the registration tree as `iso(1), member-body(2), us(840), rsadsi(113549), digest-algorithms(2), 5`.

### XML — the meta-language
*"XML is a meta-language that allows you to create specialised application-level languages for specialised client-server interactions."* A "document" here means **any structured information** — text, graphics, mathematical equations, tables and charts, executable code, interface descriptions, object meta-data (such as MPEG metadata) and much more.

> "In effect XML has become the new generation mechanism for what was originally called **electronic data interchange (EDI)**. However, XML is many times more powerful than the old EDI syntax because of its ultimate flexibility. Also, whereas originally EDI was focused around simple point-to-point file transfers, XML is supported by the more powerful web technologies."

### ASN.1 vs XML — the selection rule
> "ASN.1 and XML are neither in conflict nor in competition. They do different jobs in different ways."

| | ASN.1 | XML |
|---|---|---|
| Purpose | Specifying data communications protocols in the traditional sense | Encoding document exchanges (*"the difference is subtle, because an XML document exchange is in fact an elaborate data communications protocol"*) |
| Encoding | **Binary** | **Text** — much less efficient, greater bandwidth |
| Where it wins | **Where efficiency matters because the protocol is implemented in a real-time embedded system such as a router** | **Where the interaction is truly in the business applications layer** — greater flexibility, bandwidth is available, performance not so critical |

On the efficiency debate: *"the bandwidth revolution in which huge bandwidths have become available at low cost has rather taken away the anxiety about efficiency. It is rather like the situation where the need to fit all computer software into 512KB of memory disappeared once the price of computer memory and CPU power fell to minimal levels."*
**They nest:** because XML can embed any data including binary objects, *"you will also see ASN.1 data structures inside XML documents. A particularly relevant example is the X.509 digital certificate, which is specified in ASN.1 and binary-encoded but which will often be sent as part of an XML document."*

### The web services architecture (Figure 13-2)
> **Web Services Definition:** "'Web services' are self-contained, self-describing, modular applications that can be published, located, and invoked across the web. They perform black-box functions, which can be anything from simple requests to complicated business processes. Once a web service has been deployed other applications (and other web services) can discover and invoke the deployed service. Access to web services is through use of standard Internet protocols. The web service interface is defined strictly in terms of the messages that the web service accepts and generates."

Three architectural points:
- **The consumer** can be written in any language on any platform. *"The only constraint is that it must comply with the XML message interface to the web service."*
- **The "listener"** is a **service-agnostic** component that simply receives and sends XML messages. *"It does not interpret the messages and therefore is generic to all web services."*
- **The "business façade"** interprets the messages and is therefore **specific to the web service**. It translates between the external XML message interface and the internal interface to the business logic — the internal interface being *"whatever existing middleware architecture you happen to have to integrate your existing business applications."*

**Five components needed to implement a web services architecture:**
| Need | Component |
|---|---|
| A standard way to represent data | **XML** representation plus an **XML schema** to define data types |
| A common, extensible message format | **SOAP** supported over HTTP |
| A common, extensible service-description language | **WSDL** — Web Services Description Language, an XML-based "contract" language developed jointly by Microsoft and IBM, letting developers create and interpret contracts of what a web service is offering |
| A way to discover services located on a particular web site | **Disco** — Discovery Protocol, defining a "discovery document" retrievable from a known URL |
| A way to discover service providers | **UDDI** — Universal Description, Discovery and Integration |

**XML Schema** provides the shared vocabulary and processing rules for a new application language: *"The schema define the structure, content, syntax and semantics of the XML documents to be used… it allows the rules that have been devised by humans to be interpreted by machines."*

**SOAP** is a peer-to-peer message exchange protocol for a decentralised, distributed environment giving many-to-many connectivity. *"Fundamentally it is a stateless, one-way message paradigm"* — footnoted as *"rather like sending a holiday postcard to which no reply is expected"* — but it can be a building block for state-dependent exchanges such as request/response pairs. Structure (Figure 13-3): a **SOAP Envelope** containing a **SOAP Header** (routing and processing information, in header blocks) and a **SOAP Body** (the end-to-end payload for the final recipient, in body sub-elements). Messages travel through **SOAP Nodes** — sending, receiving and intermediary — which route according to header information.

### Trust assertions — the extension of the certificate idea
> "A trust assertion is an extension of the digital certificate concept — except that whereas a digital certificate makes assertions about the trustworthiness of an **identity**, a trust assertion has much wider applicability, allowing trusted statements to be made about **any type of information or transaction**."

Trust assertions can implement the Chapter 10 conceptual trust models. A **"trust service" is a special type of web service**, so eBusiness applications need not implement their own — but *"as with any trust brokerage, the degree of trust depends upon the credibility of the service provider, which has nothing at all to do with technology."*

**The anticipated premium trust services market:** payment gateway services; credit rating services; authentication services; transaction authorisation services; **role-based authorisation services for an entire business community**; **confidential information repositories (such as shared databases of patients' health care records) supporting trusted consortia for sharing sensitive business information**; enterprise-level authentication and role-based authorisation services.

### XML security architecture — the attack and the two countermeasures
> "The main attacks against XML-based systems are by means of **executable malicious code inserted into XML documents**."

Two architectural approaches required:
1. **All XML documents must be signed** to detect unauthorised changes, insertions, deletions or replays.
2. **XML tunnelling through conventional network-level firewalls must be controlled and managed by XML application-level firewalls.**

The warning attached: *"The XML family provides a rich combination of security mechanisms embedded within the various protocols, but if application architects and programmers fail to design for an appropriate set of security services to be implemented through these mechanisms, then there will be no security advantage. **There is a huge risk that all the common mistakes are being made all over again with these new technologies, despite the availability of the solution components.**"*

### XML firewalls
*"XML application-level firewalls are much more sophisticated than IP firewalls."*
**Design principles:** access must be controlled on the basis of a **combination** of applications (web services) requested, **APIs (functions) requested**, user identities (service requestors), XML or SOAP messages, and URLs. The firewall must decide whether a given XML message may access a specific operation or a specific web service, and must decide **according to the authorisations for the given service requestor identity and role**.
**Security services an XML firewall should provide:** authorisation against registered roles; authentication of requestor identity based on **signature verification**; role-based access controls; audit trails (event logging); administration of users, privileges and roles; **decryption and re-encryption to open up the XML documents for inspection**; and integration with directories, PKIs and single sign-on systems to support full application-level security architecture.

## Reference Tables

### The standards-making bodies and what each contributes
| Body | Nature | Security-relevant contribution |
|---|---|---|
| **ISO** (www.iso.org) | Worldwide federation of national standards bodies from **more than 140 countries**, one per country; non-governmental, established **1947** | Many international standards addressing information security and the security of information systems and networks |
| **IEC** (www.iec.ch) | The leading global body for electrical, electronic and related technologies | *"Where the activities of the IEC and ISO overlap, standards are published jointly"* — hence the **joint ISO/IEC** security standards |
| **IETF** (www.ietf.org) | Large open international community of network designers, operators, vendors and researchers, **open to any interested individual**; meets three times a year; work done in working groups organised by area, managed by **area directors (ADs)** who form the **IESG**; the **IAB** provides architectural oversight and adjudicates appeals; both chartered by **ISOC** | **19 working groups in the security area** at time of writing, including PKIX, SMIME, IPSec, TLS. Each version of a specification is published in the **RFC** series |
| **Common Criteria** (www.commoncriteria.org) | Harmonisation of security product evaluation criteria | The lineage: **TCSEC ("Orange Book")**, US, early 1980s → **ITSEC v1.2**, European Commission, 1991 (France, Germany, Netherlands, UK) → **CTCPEC v3.0**, Canada, early 1993 (combining ITSEC and TCSEC) → draft **Federal Criteria (FC) v1.0**, US, early 1993 → ISO work begun 1990 → **Common Criteria v1.0 in 1996, v2.0 in 1998**. Driven by the need for **mutual recognition of standardised security evaluation results in a global IT market** |
| **ANSI** (www.ansi.org) | US national standards body; private, non-profit | **X3 Information Processing Systems** — including cryptographic algorithms and their modes of use; **X9 Financial Services** — *"ANSI has long been a world leader in the development of standards for security of financial transactions and messaging"* |
| **BSI** (www.bsi-global.com) | UK national standards body | **BS7799 "A Code of Practice for Information Security Management"**, moved internationally as **ISO/IEC 17799**; **BS15000 "IT Service Management"** → ISO/IEC 15000; and, under development at time of writing, **PAS 56 "Guide to Business Continuity Management"** |
| **ITU** (www.itu.int) | Inter-governmental, established **1865** when international telegraphy was new; a UN specialised agency since 1947; represents more than 170 countries | Until 1993 had two standards arms, **CCITT** and **CCIR** — hence older labels. *"Just to add to the confusion, there is also dual labelling of many standards with both ISO/IEC numbers and ITU numbers — which are different."* The **CCITT X-series** including **X.400, X.500 and X.509** is very relevant to information security |
| **IEEE** (www.ieee.org) | Non-profit technical professional association, **more than 377,000 members in 150 countries**; nearly 900 active standards with 700 under development | Committee **802.10** — Standard for Interoperable LAN Security (**SILS**) |
| **ISACA** (www.isaca.org) | **More than 26,000 members in over 100 countries**; founded 1969; administers **CISA** (earned by more than 29,000 professionals) and the then-new **CISM**; established the **IT Governance Institute** in 1998 | **CobiT** — Control Objectives for Information and Related Technology, then in its third edition |
| **OMG** (www.omg.org) | Founded April 1989 by eleven companies; ~800 members; Model Driven Architecture, CORBA, UML, XMI, MOF | **CORBA CSIv2** — Common Secure Inter-Operability Version 2 |
| **OASIS** (www.oasis-open.org) | Not-for-profit global consortium; **more than 600 corporate and individual members in 100 countries**; founded **1993 as SGML Open**, renamed 1998; jointly sponsors **ebXML** with the United Nations; operates xml.org | Worldwide standards for security, web services, XML conformance, business transactions. **The primary body for XML security development** — SAML, WS-Security, XACML, X-TASS |
| **W3C** (www.w3.org) | Created **October 1994**; around 450 member organisations | XML Encryption, XML Signature, and the web's common protocols |
| **OECD** (www.oecd.org) | 30 member countries, active relationships with some 70 others | **"Guidelines for the Security of Information Systems and Networks: Towards a Culture of Security", published 2002** |
| **US Federal Government** (www.itl.nist.gov/fipspubs/) | **FIPS PUB** developed and published by **NIST**; plus DOD standards | Primarily for government systems, *"However, because vendors of government systems also sell into the commercial world, these standards often become important outside the immediate government domain"* |
| **Standards Australia / Standards New Zealand** | SA founded 1922 as the Australian Commonwealth Engineering Standards Association; incorporated as Standards Australia International Limited in 1999. SNZ is the trading arm of the Standards Council under the Standards Act 1988 | Useful standards on managing the security of information, **risk management** and **corporate governance**, some harmonised across both countries |
| **JISC** (www.jisc.go.jp/eng/) | Japanese Industrial Standards Committee | Establishment and maintenance of **JIS**, certification accreditation, measurement standards |
| **ECMA** (www.ecma-international.org) | Industry association founded 1961; more than 335 standards and 85 technical reports, **more than 40% also adopted as international standards**; standards are freely copyable without restriction | **Project Sesame** — an architecture for a **centralised authentication service**, with member manufacturers building products on it |
| **ETSI** (www.etsi.org) | Not-for-profit, based in Sophia Antipolis, southern France; **768 members from 55 countries** | European telecommunications standards; members determine the work programme, keeping it aligned with market needs |
| **EEMA** (www.eema.org) | European Forum for Electronic Business, formed **March 1987**; international, non-profit | Independent forum covering IT security (trusted service provision, digital certificates, PKI), EDI and XML, e-mail, directories, the Internet and web, e-commerce best practices, wireless and unified messaging |
| **Wi-Fi Alliance** (www.weca.net) | Non-profit international association formed **1999**; **205 member companies**, **915 products certified** since March 2000 | Certifies interoperability of WLAN products based on **IEEE 802.11** |
| **TCG** (www.trustedcomputinggroup.org) | Computer, device and chip manufacturers, software vendors and others; **replaced the Trusted Computing Platform Alliance (TCPA)**, adopting its specifications as initial ones | Open standards for trusted computing hardware building blocks and software interfaces across PCs, servers, PDAs and digital telephones; the **Trusted Platform Module (TPM) specification, version 1.1b** |
| **ISF** (www.securityforum.org) | International Security Forum — **over 250 leading organisations**; over 13 years has invested **more than $40 million** in best-practice material | *"The most comprehensive and integrated set of reports anywhere in the world about the process of managing information risk"* |

**Vendor standards.** *"In the end most international standards are driven by the large vendors, because they are the ones who are most represented on the standards-making bodies."* Sometimes a vendor forges ahead alone because *"the formal standards-making bodies take years to agree and they have a more urgent agenda (although the informality of the IETF RFCs and Internet Drafts avoids this problem completely)."* Famous examples: **PKCS from RSA Security**, which later became the foundation for many Internet RFCs and Drafts; **SSL from Netscape**, later published as an Internet Draft.

### Internal security standards you must write yourself
- **Platform security standards** for each type of platform in use, stating how to configure and operate them.
- **Cryptographic standards** — which algorithms and protocols for which applications, usually aligned with external standards.
- **Password management standards** — length, syntax, frequency of change.
- **Physical security standards** for various types of site, building and installation.
- **Document standards** for all security-related documents, including intranet publications.
- **Secure communications interface standards** for connecting applications to the enterprise infrastructure.
- **User interface standards** for providing consistent user login.

### Table 13-1: Security Tools and Products (the component taxonomy)
| Component type | Common features / mechanisms |
|---|---|
| Anti-piracy tools | Preventing the illegal copying and distribution of software |
| Anti-theft devices | Preventing theft of equipment items such as PCs |
| Anti-virus scanners | Scanning for known viruses and other malicious software and repairing damaged files — *"although the repair may not be perfect and therefore may not be the correct way to proceed"* |
| Biometric devices | Personal authentication based on measurement of a bodily feature — fingerprint, retina pattern, facial geometry |
| Boot-protection software | Preventing booting of a PC from a diskette to get unauthorised access to the hard drive |
| BCP and DRP tools | Supporting collection and management of planning information |
| CCTV monitoring | Physical site surveillance |
| Computer forensics tools | Recovering deleted data and piecing together a history of activity |
| Content filtering for e-mail / for web browsing | Detecting and filtering out unacceptable content |
| Cryptographic hardware | High-performance cryptographic processing, high-security key storage, **secure time source**, random number generation for key management, tamper-resistant enclosures |
| Cryptographic software tool-kits | Run-time libraries for data encryption, authentication, digital signatures and certificate processing |
| Data back-up management systems | Copying and storage management, restoration to a previous business position |
| Directory products | Providing directory services |
| Document safes | Protecting documents from theft and fire damage |
| E-mail encryption and authentication products | Privacy and authentication for e-mail messages |
| Enterprise security management tools | Managing a wide range of security services across multiple platforms |
| Fault-tolerant computing solutions | Resilient platforms that survive component failure |
| File encryption products | Encrypting files for transmission or for storage |
| Firewalls | Filtering network traffic by source, destination and content to allow only authorised traffic |
| Intrusion detection systems | Looking for unauthorised activity from intruders **both in the network and on host platforms** |
| LAN security products | Security functionality in local area networks |
| Operating platforms | Logical access control and integrity protection |
| Personal authentication tokens and devices | Multi-factor authentication of users |
| Physical security alarms | Intruder alarms and fire alarms in buildings and computer suites |
| PKI software | Digital certificate management and the cryptographic services it supports |
| Risk assessment tools | Software packages to capture and process risk data |
| Role-based access control solutions | Centralised RBAC management and authentication of users |
| Secure middleware products | Secure node-to-node communications and an **API for applications to call security services** |
| Security auditing tools | Automated inspection tools to check the configuration of an operating platform or application |
| Security shells | Add-on software providing additional levels of access control to standard operating systems |
| Single-sign-on authentication service solutions | Centralised authentication servers integrating distributed applications, providing an authentication front end with SSO |
| Smart cards | **A self-contained computer on a plastic card with its own on-board authentication and access control functions** |
| Software licence management tools | Managing distribution of licensed software to ensure compliance |
| Uninterruptible power supplies | Protecting against electrical power failure |
| VPN products | Virtual private networks built using **IPSec or SSL** |
| Vulnerability scanning tools | Looking for holes in network or host configurations |
| Wireless security products | Preventing eavesdropping and authenticating nodes |

### The XML security standards family
| Standard | What it does |
|---|---|
| **XML Encryption** (W3C) | Techniques for using XML to represent encrypted XML and other data. Permits encryption of **an entire XML document, specific elements, or just specific element content**. Any algorithm may be used, specified in the `<EncryptionMethod>` element |
| **XML Signature (XML-SIG)** (W3C) | XML-based digital signature for data of **any** type. The signed data may reside within the same XML document as the signature **or elsewhere**. The signature is a short digest, optionally encrypted. Any signature algorithm may be used, specified in `<SignatureMethod>` |
| **SOAP Extensions: Digital Signature** | Open-standard method for digitally signing SOAP messages; based on XML Signature or, if required, non-XML signature standards |
| **XKMS** (Microsoft + Verisign) | XML Key Management Specification — integrates PKI and digital certificates with XML applications. Two parts: **X-KISS** (XML Key Information Service Specification), a protocol for a trust service that resolves public key information in XML-SIG elements; **X-KRSS** (XML Key Registration Service Specification), letting a client delegate the processing of "key information" elements to a dedicated web service, *"protecting the main application from the potential complexity of handling a variety of underlying PKI standards, including X.509/PKIX, SPKI and PGP"* |
| **S2ML** | Security Service Markup Language — a common language for exchanging **authentication, authorisation and user profile information (ACLs)** in support of B2B and B2C transactions, independent of platform and transport protocol. Uses **URIs as unique names for entities — and a URI can also identify a role**, as in RBAC. Provides syntax for **trust assertions** |
| **SAML** (OASIS) | Security Assertion Markup Language — a framework for exchanging security information between business partners over the Internet; *"similar to and modelled on the earlier S2ML, but it is not the same."* Benefits: works with any industry-standard transport (HTTP, SMTP, FTP) or XML document-exchange framework (SOAP); **interoperability between end users, service providers and brokers without any of them changing their local security solutions**; **single sign-on across multiple web sites under multiple ownership** |
| **WS-Security** (OASIS) | Built on XML Encryption and XML Signature; enhances SOAP with confidentiality and integrity protection and security-information exchange, using **security tokens holding claims**, similar to SAML assertions |
| **XACML** (OASIS) | extensible Access Control Markup Language — an XML-based common **security policy language for expressing information system access policy** |
| **XBRL** | extensible Business Reporting Language — a standard interface for **automated reporting of business and financial information**; of interest for compliance reporting to industry regulators and *"an important element in the construction of highly integrated operational risk management systems"* (Chapter 15, Figure 15-9). Advantages: platform-independent, globally inter-operable, less expensive and more efficient than manual methods |

### SAML vs WS-Security — the comparison the book draws
| | SAML | WS-Security |
|---|---|---|
| Weight | Heavier — more infrastructure | **Lighter**, far less infrastructure |
| Functional richness | Richer | Less rich; **does not provide single sign-on** |
| Confidentiality | **Does not support it** — relies on SSL or TLS at a lower layer | **Includes confidentiality at the application layer using XML Encryption** |

**The flaw the book identifies in SAML's approach:** *"This approach is deeply flawed insomuch as the application has no control over and no signalling channel with the SSL/TLS level of the stack and cannot tell whether or not the confidentiality service is switched on."*
The verdict: *"At the time of writing WS-Secure and SAML are competing for dominance. It is possible that one will prevail over the other, but a much more attractive outcome would be that they will merge into one standard, bringing together the strong points of both and letting go of the weaknesses."*

### What XACML resolves
A method for **combining individual rules and policies from distributed entities into a single policy set** applying to a particular decision request. *"The ability to handle this level of complexity is a huge advantage, since the intersection of a widely varying set of access rules can be very confusing."* Complexities resolved: subjects acting in different capacities; authorisation decisions based on the **combined attributes of subject, resource (object), environment and the action or function requested** (Figure 13-4); assessing the contents of an information resource; and dealing with distributed policy components.

### Figure 13-5: The infrastructure stack and security-related protocols
| Layer | Security protocols |
|---|---|
| Business Applications | — |
| Application security | **Message Security** (SAML, S2ML, WS-Security, XACML, SOAP Extensions) alongside **Object Security** (CORBA, Java) |
| XML security primitives | XML Digital Signatures, XML Encryption, XML Key Management |
| Application transport | S-HTTP; HTTP, FTP, SMTP |
| Session/transport security | **SSL & TLS** |
| Transport | TCP / UDP |
| Network | **IPSec**; IP |
| Sub-network | — |

## Worked Example

### The SSL padlock, cross-examined
The chapter's signature artifact is a **question-and-answer interrogation** of what SSL actually delivers, aimed squarely at *"a popular myth that once you see the padlock icon on your browser screen telling you that you are communicating with the server over an SSL-secured link, everything is nice and secure and there are no worries."*

**What the padlock actually means:** SSL provides a **strongly encrypted pipe** between server and client browser, so data in transit is safe from eavesdroppers. During session establishment an authentication handshake ensures the server has a valid SSL certificate, so **server-side authentication is automatic**. SSL *can* be configured for client-side authentication *"but it is rarely implemented, and in all cases where you use SSL to access a public Internet site it is not implemented."* So: **the padlock tells you that you have established a strongly encrypted pipe directly to a server that has a valid SSL certificate.** Nothing more.

| Question | The book's answer |
|---|---|
| **Does SSL help you to trust the server to be run by an honest, well-managed organisation?** | **No** — "pretty much anyone can apply for a certificate, pay the money and get one. To run an SSL server you will merely have to prove that you exist and have that name, but the certificate authority will register you without checking anything about your honesty. **There is no implied business trust in SSL certificates**" |
| **Is your data kept confidential during transmission to the server?** | **Yes** — but once it arrives it may be stored on the server for some time, and the server may be easily hacked from outside to allow theft. "SSL only protects the transmission along the pipe" |
| **Can you be sure the server belongs to the organisation you believe it represents?** | **Only if you are very careful about checking the name in the URL** (cross-referenced to the IBFS Internet Bank case study in Chapter 9). "**People with fake names can get SSL certificates**" |
| **Will your transactions be protected from unauthorised changes?** | **Only whilst they are in the pipe.** Malicious software already on your client PC could make unauthorised changes before transmission; and once the data emerges at the server end the SSL protection stops |
| **Will SSL provide non-repudiation of a business transaction?** | **"Definitely NOT!** All the SSL mechanisms provide transitory protection during transmission. Once the data arrives, any encryption is stripped off and discarded, and any implied authentication is lost with it. **There is no audit trail of digital signatures or message authentication codes that can be retrospectively matched to business transactions for proof of sending**" |
| **Will the fact that SSL was used help resolve disputes about the business transactions?** | **"Sorry, no.** There is no audit trail of security on the transaction. **There is no way of even proving that the SSL was switched on and working at the time of the transaction, because the business application has no visibility of that**" |
| **Does SSL protect against fraud?** | **No** — "the fraud is usually perpetrated by someone at the client end or by someone at the server end, and SSL cannot control what these parties do" |
| **Why then is SSL promoted as the solution to eBusiness security requirements?** | **"Because most people do not understand its limitations.** It provides a strong encrypted pipe that protects against eavesdropping on and alteration of the transmissions. That is all it does, and it does it very well. However, securing eBusiness transactions takes much, much more than that" |

**The conclusion:** *"whilst SSL is not the complete solution to securing eBusiness applications, it is an important component. However, it is only one component in what has to be a holistic security architecture, addressing all the potential threats, not just those associated with transmission."*

### The same cross-examination applied to IPSec
**What IPSec is:** a large group of standards describing how cryptographic security is integrated into the IP packet layer. Two basic approaches — **Encapsulating Security Payload (ESP)** and **IP Authentication Header (AH)**. Key management is handled by the **Internet Key Exchange protocol (IKE)**. Two modes: **transport mode reveals the original IP source and destination addresses; tunnel mode hides them and reveals only the addresses of the IPSec gateway** — so tunnel mode provides *"a limited degree of traffic flow confidentiality in addition to packet payload content confidentiality."* No mandatory encryption algorithm is specified; vendor support is the only real constraint. Authentication uses **keyed hashing (HMAC)**, with MD5, SHA-1 and SHA-256 the then-supported hashing algorithms.

**What it does deliver:** *"It is certainly an excellent technology for building a virtual private network. As such it will provide blanket data confidentiality and perhaps limited traffic flow confidentiality down in the network packet layer. It will also guarantee the authenticity of all nodes in the IPSec network, preventing any rogue nodes from being introduced."*

**What it does not:** *"like SSL, IPSec has no way of communicating with the application layer, and so it offers no direct assurance or trust to the application. The application has no way of knowing whether or not IPSec is switched on and working, or whether all or only part of the network is encrypted and authenticated. **Specifically IPSec does not provide any authentication or non-repudiation for applications.** IPSec is a technology for providing network security, and is not a technology for providing application security."*

### Three protocols people confuse
| Protocol | What it actually is |
|---|---|
| **HTTP** | The underlying web client-server protocol. Successful for two reasons — **simplicity** (a user types a URL and navigates transparently, seeing the logical information structure of the web but not its physical structure; a site is built with a simple markup language and low-cost power tools; thousands of ISPs will host it competitively) and **ubiquity** ("by setting up a web server you automatically join a global community"). It is also *"the basic workhorse"* transport for every XML security protocol in the previous section |
| **S-HTTP** | **A modified version of HTTP**, described in an Internet Draft from June 1998. Provides mechanisms for confidentiality, authentication and integrity; **not tied to any particular cryptographic system, key infrastructure, key certification scheme or cryptographic format**. Allows HTTP messages to be encapsulated with encryption, signing or MAC-based authentication — and **the encapsulation can be recursive**, applying several successive security transformations. Includes header definitions for key transfer, certificate transfer and similar administrative functions |
| **HTTPS** | **"Do not be confused! HTTPS is NOT the same as S-HTTP."** HTTPS is simply **unmodified HTTP run over either SSL or TLS** |

**SSL vs TLS:** SSL was developed by Netscape and is a **proprietary protocol**, though published and widely implemented; SSL version 3 was published in **November 1996** as an Internet Draft. **TLS is the official non-vendor IETF replacement, specified in RFC 2246, 1999.**

**Two supporting protocols:**
- **DNSSec (RFC 2535)** — needed *"to complement the security mechanisms within IPSec… otherwise the overall security of the network can be compromised beyond the control of IPSec."* It applies digital signatures (RSA or DSA) to authenticate DNS requests and responses and provides authenticated storage and distribution of public keys, both for its own purposes and for other network-level secured protocols. **DNSSec does not provide confidentiality for DNS traffic.**
- **SASL (RFC 2222)** — Simple Authentication and Security Layer, *"a generic, flexible security protocol that adds authentication support to connection-based protocols through the use of registered mechanisms."* Servers may support one or more registered mechanisms, in which case a secure connection can be negotiated. **SASL also allows a proxy client with one identity and credentials to authorise on behalf of another entity.**

### Security step timing and sequencing (the component When cell)
Timing and sequencing is **primarily driven by business requirements** — user expectations, business deadlines, volume throughput — but in this cell *"you must deal with the nuts and bolts of how that timing and sequencing is to be achieved in practice. Much of this will depend upon the performance and efficiency of the various components that you have assembled."*
**The detailed design task** focuses on how each component performs and **how the timing of their individual operations can be interlocked** to deliver the business requirements. The cross-reference given is the Chapter 10 case study on Time Performance Issues, which discusses designing a process that involves decryption of database records.
**The key insight:** *"it is the interaction between components that you must consider. The actual sequencing of component operations may be different from the logical business flow, simply to gain better performance efficiency."*

## Mental Models
- **Standards exist so components plug together.** The whole component layer is an inter-operability problem, and the answer is compatible — preferably internationally standardised — interfaces.
- **Binary for embedded, text for applications.** ASN.1 where the router needs efficiency; XML where the business application needs flexibility. And ASN.1 nests inside XML.
- **The listener is generic, the façade is specific.** Every web service has the same message plumbing and its own translation layer to the business logic.
- **A trust assertion generalises a certificate** from "this identity is trustworthy" to "this statement about any information or transaction is trustworthy."
- **The pipe is not the transaction.** SSL and IPSec both secure a channel; neither can tell the application whether it is on, and neither leaves an audit trail. That is why network security is not application security.
- **New technology, same old mistakes.** The XML family ships the mechanisms; nothing forces architects to use them.

## Anti-patterns
- Treating the SSL padlock as evidence of a trustworthy counterparty, of end-to-end confidentiality, of transaction integrity, of non-repudiation or of fraud protection.
- Confusing HTTPS with S-HTTP.
- Relying on SSL/TLS at a lower layer for confidentiality when the application has no signalling channel to it and cannot tell whether it is switched on (the SAML flaw).
- Using IPSec as if it were an application security mechanism.
- Deploying IPSec without also securing DNS lookups.
- Unsigned XML documents — leaving insertion, deletion, change and replay undetectable.
- Letting XML tunnel through a conventional network-level firewall that cannot inspect it.
- Selecting components with unique, non-standard interfaces, then discovering systems integration is very difficult.
- Assuming an anti-virus scanner's automatic repair is the correct course of action — *"the repair may not be perfect and therefore may not be the correct way to proceed."*
- Expecting a printed list of current standards to stay useful — *"they go out of date very quickly."*

## Key Takeaways
- Component = tradesman's view: standards, products, tools, protocols, data structures — generic principles, never a vendor recommendation.
- Inter-operability comes from compatible, standardised interfaces; agreeing on the exchanged **data structures and protocols** matters most.
- **ASN.1** (binary, ISO/IEC 8824, with BER/DER encoding and OBJECT IDENTIFIERs) and **XML** (text, meta-language, EDI's successor) are the two foundational syntax standards, and they nest rather than compete.
- Twenty-plus standards bodies matter; know which one owns what — ISO/IEC for the joint security standards, IETF for RFCs, OASIS for XML security, W3C for XML primitives, ANSI X9 for financial security, BSI for BS7799 → ISO 17799, TCG for the TPM.
- **Write your own internal standards too** — platform, cryptographic, password, physical, document, communications interface and user interface.
- Table 13-1 is the generic product taxonomy — roughly forty component types with their standard feature sets.
- Web services need XML + schema, SOAP, WSDL, Disco and UDDI; security is layered on via XML Encryption, XML Signature, XKMS, S2ML/SAML, WS-Security, XACML and XBRL.
- SAML and WS-Security were competing; SAML has SSO but no confidentiality, WS-Security has application-layer confidentiality but no SSO.
- **SSL and IPSec secure pipes, not transactions** — no application visibility, no audit trail, no non-repudiation.
- XML firewalls decide on the combination of web service, API, identity, message and URL — and must decrypt to inspect.
- The component When cell is about interlocking component timings; the optimal sequence may deliberately differ from the logical business flow.

## Connects To
- [Chapter 3 — Security Architecture Model](ch03-security-architecture-model.md) — the component row of the SABSA Matrix.
- [Chapter 7 — Using This Book as a Practical Guide](ch07-using-this-book-as-a-practical-guide.md) — Figure 7-11, the component layer deliverables.
- [Chapter 9 — Contextual Security Architecture](ch09-contextual-security-architecture.md) — the IBFS Internet Bank case study behind "people with fake names can get SSL certificates."
- [Chapter 10 — Conceptual Security Architecture](ch10-conceptual-security-architecture.md) — the trust models that trust assertions implement, the access control model XACML expresses, the secure-pipe argument, and the network-security-vs-application-security rule.
- [Chapter 12 — Physical Security Architecture](ch12-physical-security-architecture.md) — the mechanisms these products and protocols instantiate, and the TCG/LaGrande hardware discussion.
- [Chapter 15 — Operational Risk Management](ch15-operational-risk-management.md) — the integrated operational risk management systems XBRL feeds (Figure 15-9).
- [Chapter 16 — Assurance Management](ch16-assurance-management.md) — Common Criteria and product evaluation as an assurance instrument.
