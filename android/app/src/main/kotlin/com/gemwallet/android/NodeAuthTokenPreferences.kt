package com.gemwallet.android

import com.gemwallet.android.data.services.gemapi.DeviceToken
import com.gemwallet.android.serializer.fromJson
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemPreferences

private const val AUTH_TOKEN_KEY = "authToken"

internal fun GemPreferences.authToken(): DeviceToken? = get(AUTH_TOKEN_KEY).fromJson()

internal fun GemPreferences.setAuthToken(token: DeviceToken) = set(AUTH_TOKEN_KEY, token.toJson())
