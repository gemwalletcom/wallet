package com.gemwallet.android.data.services.gemapi

import kotlinx.serialization.Serializable

@Serializable
data class DeviceToken(
    val token: String,
    val expiresAt: ULong,
)
