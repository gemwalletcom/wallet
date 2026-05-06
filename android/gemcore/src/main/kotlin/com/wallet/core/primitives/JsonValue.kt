package com.wallet.core.primitives

import com.gemwallet.android.serializer.JsonAsStringSerializer
import kotlinx.serialization.Serializable

typealias JsonValue = @Serializable(with = JsonAsStringSerializer::class) String
