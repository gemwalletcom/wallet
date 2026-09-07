package com.gemwallet.android.domains.confirm

import com.gemwallet.android.serializer.packRoutePayload
import com.gemwallet.android.serializer.unpackRoutePayload
import uniffi.gemstone.GemTransferData

fun GemTransferData.pack(): String? = packRoutePayload()

fun unpackTransferData(packed: String): GemTransferData? = unpackRoutePayload(packed)
