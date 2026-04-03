package com.gemwallet.android.ext

import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.LinkType

val AssetLink.linkType: LinkType?
    get() = when (name.lowercase()) {
        "twitter" -> LinkType.X
        else -> LinkType.entries.firstOrNull { it.string == name.lowercase() }
    }
