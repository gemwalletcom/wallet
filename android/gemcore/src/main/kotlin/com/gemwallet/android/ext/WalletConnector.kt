package com.gemwallet.android.ext

import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.ApplicationMetadata
import uniffi.gemstone.GemApplicationMetadataService

val ApplicationMetadata.shortName: String
    get() = GemApplicationMetadataService().use { it.shortName(toJson()) }
