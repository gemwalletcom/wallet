package com.gemwallet.android.ext

import kotlinx.coroutines.CancellationException

suspend fun <T> runCatchingCancellable(block: suspend () -> T): Result<T> = try {
    Result.success(block())
} catch (err: CancellationException) {
    throw err
} catch (err: Throwable) {
    Result.failure(err)
}
