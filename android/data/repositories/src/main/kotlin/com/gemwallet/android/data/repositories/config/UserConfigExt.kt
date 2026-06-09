package com.gemwallet.android.data.repositories.config

import com.gemwallet.android.ext.hasPerpetualsSupport
import com.gemwallet.android.model.Session
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine

fun UserConfig.showPerpetuals(session: Flow<Session?>): Flow<Boolean> =
    combine(session, isPerpetualEnabled()) { current, enabled ->
        enabled && current?.wallet?.hasPerpetualsSupport == true
    }
