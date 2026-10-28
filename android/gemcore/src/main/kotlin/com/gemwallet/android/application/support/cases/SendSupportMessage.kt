package com.gemwallet.android.application.support.cases

import com.wallet.core.primitives.SupportMessage

interface SendSupportMessage {
    suspend fun sendText(content: String)

    suspend fun sendImages(images: List<ByteArray>)

    suspend fun retry(message: SupportMessage)
}
