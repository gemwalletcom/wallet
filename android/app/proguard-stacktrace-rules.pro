# Preserve source line numbers for store releases.
#
# Reproducible/F-Droid builds intentionally skip LineNumberTable because R8 9.2.14
# produced non-deterministic outline and residual-signature position metadata across
# two clean Linux builds.
-keepattributes LineNumberTable
