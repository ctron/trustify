# Glossary of Understanding

## Entities

### Vulnerability

A vulnerability is mostly, primarily a *name* that is used to ensure all advisories are discussing the same thing.
Generally, to this point, most vulnerabilities come from the CVE Project, with the format of `CVE-2024-1234`.

Within the database, generally a vulnerability is added as a side effect of an advisory mentioning it.

A *CVE Record* from NIST/NVD is a low-value advisory that is generally the first discovered advisory that mentions a vulnerability.

### Advisory

An advisory is an opinion about a vulnerability.

These opinions include the context to which the opinions apply.
These opinions include evaluation of the severity and scoring of a vulnerability within that context, such as CVSS scores.

As mentioned above, a *CVE Record* from the CVE Project is a low-value advisory that mentions the vulnerability and provide a base opinion about it.
It may include CVSS scores, within the context of the abstract origin containing the vulnerability.
This may be simply in reference to the vulnerability *as it exists in source-code form*.

Other, more-involved stakeholders (product vendors, upstream project owners) may issue *additional* advisories.
These opinions may be in reference to *concrete* shipped products, contextualized to how the vulnerable code is *actually used*.

### SBOM

An SBOM is a source-of-someone's-truth about "what's inside it?", so
everything in our DB is ultimately sourced from some
source-of-truth. We can't really say definitively "product X is
composed of A1, A2 + A3". Instead, we can have multiple simultaneous
statements -- SBOM's -- from multiple people saying "product X is
claimed by Bob to be A1 + A2" and "product X is claimed by Jim to be
A1 + A97". So an SBOM is the entity to track the origin of the
supposed "evidence" of assertional statements about products... about
packages... about vulnerabilities...

### Package

A package is an atomic artifact or component.
Packages may be addressed using pURLs.
A package may be described by an SBOM describing how it is created and its contents.
A package may certainly contain other packages (e.g. shading one Java jar into another).
A package may also be the sole member of a Product (`UBI-8.0.13-x86.oci` may be the singular package within the "UBI 8.0.13-x86" product).
A package is one step more abstract than an *artifact*.

#### pURL

Package URLs (pURLs) are possibly ambiguous names applied to packages.
A simple pURL such as `pkg:maven/org.apache/log4j@1.2.3` may or may not refer to a unique artifact.
With additional qualifiers, it is possible to produce a URI that asserts uniqueness, such as `pkg:maven/org.apache/log4j@1.2.3?repository_url=repo.jboss.com`.
Without additional qualifiers, the implicit aspects (such as `repository_url`) must be taken into account.
For instance, an unqualified `pkg:maven` pURL *implies* "the jar from Maven Central, and none other".

### Product

A product is a *named collection of 1 or more packages* for a concrete shippable thing.

Products may be addressed using CPEs or some other future identification method.
A product may be described by an SBOM describing its components, which may be other products or packages, or their SBOMs.

> [!NOTE]
> Given ProdSec definitions, grouping of Products may need to occur within some sense of Product Versions, or Product Streams.

`RHEL8` may be a *product stream*.
`RHEL 8.2.03 PowerPC` may be a concrete *product* distinct from `RHEL 8.2.03 AArch64`.

#### CPE

A CPE is a "Common Product Enumeration" from the NIST organization.
CPEs are self-assigned, but registered occasionally with NIST.
CPEs describe the vendor, the product, the version, target architecture, etc.
CPEs may also be non-fully specified, to use as pattern-matching.
For instance, "All versions of RHEL 8.2.013, regardless of platform", or if more fully-specified, could imply "All versions of RHEL 8.x on AArch64".

> [!NOTE]
> CPEs are somewhat contentious, and used enough for us to not ignore, but not used enough to be a pivotal definition of "product" for any users of Trustify.

### Artifact

For a given *package*, there may be zero or more instances of that package.
Given `log4j-1.2.3.jar`, seventeen different people could compile the same source with the same arguments, and still end up with 17 distinct Java jars (due to non-reproducible builds).
Each is an artifact of the *same* package.
Each may (will probably) have its own SHA-256 related to it.

Consider an *artifact* to be a concrete *instance* of a package.
