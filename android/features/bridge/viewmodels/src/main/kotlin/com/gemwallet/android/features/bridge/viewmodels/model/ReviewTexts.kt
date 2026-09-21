package com.gemwallet.android.features.bridge.viewmodels.model

import android.content.Context
import com.gemwallet.android.ui.R

data class ReviewTexts(val app: String, val wallet: String, val viewFullMessage: String) {
    constructor(context: Context) : this(
        app = context.getString(R.string.wallet_connect_app),
        wallet = context.getString(R.string.common_wallet),
        viewFullMessage = context.getString(R.string.sign_message_view_full_message),
    )
}
