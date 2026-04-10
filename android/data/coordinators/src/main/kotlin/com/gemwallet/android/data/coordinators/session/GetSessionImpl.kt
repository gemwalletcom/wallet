package com.gemwallet.android.data.coordinators.session

import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.model.Session
import kotlinx.coroutines.flow.StateFlow

class GetSessionImpl(
    private val sessionRepository: SessionRepository,
) : GetSession {
    override fun getSession(): StateFlow<Session?> = sessionRepository.session()
}
