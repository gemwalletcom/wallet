package com.gemwallet.android.ext

import com.wallet.core.primitives.ApplicationMetadata
import uniffi.gemstone.GemApplicationMetadataService

val ApplicationMetadata.shortName: String
    get() = GemApplicationMetadataService().use { it.shortName(toGem()) }

val ApplicationMetadata.iconUrl: String?
    get() = GemApplicationMetadataService().use { it.iconUrl(toGem()) }

val ApplicationMetadata.host: String
    get() = GemApplicationMetadataService().use { it.host(toGem()) }
