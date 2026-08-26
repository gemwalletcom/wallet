package com.gemwallet.android

import com.gemwallet.android.data.services.gemapi.DeviceToken
import com.gemwallet.android.serializer.fromJson
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemPreferencesStore

private const val AUTH_TOKEN_KEY = "authToken"

internal fun GemPreferencesStore.authToken(): DeviceToken? = get(AUTH_TOKEN_KEY).fromJson()

internal fun GemPreferencesStore.setAuthToken(token: DeviceToken) = set(AUTH_TOKEN_KEY, token.toJson())
