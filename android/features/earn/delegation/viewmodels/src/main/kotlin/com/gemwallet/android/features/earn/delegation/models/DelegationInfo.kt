package com.gemwallet.android.features.earn.delegation.models

import com.gemwallet.android.model.text
import uniffi.gemstone.GemDelegationDetails
import uniffi.gemstone.GemValidatorRow

class HeadDelegationInfo(private val details: GemDelegationDetails, private val validator: GemValidatorRow) {

    val iconUrl: String
        get() = validator.imageUrl

    val iconPlaceholder: String
        get() = validator.placeholder

    val cryptoFormatted: String by lazy { details.balance.text() }

    val fiatFormatted: String by lazy { details.fiat?.text().orEmpty() }
}
