package com.gemwallet.android.domains.confirm

import com.gemwallet.android.serializer.packRoutePayload
import com.gemwallet.android.serializer.unpackRoutePayload
import uniffi.gemstone.GemTransferData

data class ConfirmTransferInput(val data: GemTransferData)

fun GemTransferData.pack(): String? = packRoutePayload()

fun ConfirmTransferInput.pack(): String? = data.pack()

fun unpackConfirmTransferInput(packed: String): ConfirmTransferInput? = unpackTransferData(packed)?.let(::ConfirmTransferInput)

fun unpackTransferData(packed: String): GemTransferData? = unpackRoutePayload(packed)
