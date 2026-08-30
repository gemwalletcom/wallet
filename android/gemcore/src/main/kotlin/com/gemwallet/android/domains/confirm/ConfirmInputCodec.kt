package com.gemwallet.android.domains.confirm

import com.gemwallet.android.serializer.packRouteString
import com.gemwallet.android.serializer.unpackRouteString
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemTransferService

class ConfirmInputCodec(
    private val transferService: GemTransferService,
) {
    fun pack(input: GemConfirmInput): String? =
        runCatching { transferService.encodeConfirmInput(input).packRouteString() }.getOrNull()

    fun unpack(packed: String): GemConfirmInput? =
        runCatching { transferService.decodeConfirmInput(packed.unpackRouteString()) }.getOrNull()
}
