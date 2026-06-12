package com.gemwallet.android.features.nft.presents

import com.gemwallet.android.ext.secondsToMillis
import com.wallet.core.primitives.NFTAttribute
import com.wallet.core.primitives.NFTAttributeType
import java.text.DateFormat
import java.util.Date

internal data class NftAttributeUIModel(
    val name: String,
    val value: String,
) {
    constructor(
        attribute: NFTAttribute,
        dateFormat: DateFormat = DateFormat.getDateInstance(DateFormat.MEDIUM),
    ) : this(
        name = attribute.name,
        value = when (attribute.valueType ?: NFTAttributeType.String) {
            NFTAttributeType.Timestamp -> attribute.value
                .toLongOrNull()
                ?.let { dateFormat.format(Date(it.secondsToMillis())) }
                ?: attribute.value
            NFTAttributeType.String -> attribute.value
        },
    )
}
