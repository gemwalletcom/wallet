package com.gemwallet.android.application.session.coordinators

import com.gemwallet.android.model.Session
import kotlinx.coroutines.flow.StateFlow

interface GetSession {
    operator fun invoke(): StateFlow<Session?>
}
