package com.gemwallet.android.data.services.gemstone.config

import com.gemwallet.android.model.Session
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine

fun UserConfig.showPerpetuals(session: Flow<Session?>): Flow<Boolean> =
    combine(session, isPerpetualEnabled()) { current, _ ->
        current?.wallet?.let(::showPerpetuals) ?: false
    }
