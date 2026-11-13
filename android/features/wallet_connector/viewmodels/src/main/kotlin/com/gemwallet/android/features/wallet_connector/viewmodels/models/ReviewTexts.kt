package com.gemwallet.android.features.wallet_connector.viewmodels.models

import android.content.Context
import com.gemwallet.android.ui.R

data class ReviewTexts(val viewFullMessage: String) {
    constructor(context: Context) : this(
        viewFullMessage = context.getString(R.string.sign_message_view_full_message),
    )
}
