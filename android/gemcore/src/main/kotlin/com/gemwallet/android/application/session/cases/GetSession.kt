package com.gemwallet.android.application.session.cases

import com.gemwallet.android.model.Session
import kotlinx.coroutines.flow.StateFlow

interface GetSession {
    operator fun invoke(): StateFlow<Session?>
}
