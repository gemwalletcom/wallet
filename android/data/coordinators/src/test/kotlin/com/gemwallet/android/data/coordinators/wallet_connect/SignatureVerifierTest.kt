package com.gemwallet.android.data.coordinators.wallet_connect

import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.junit.runners.Parameterized
import java.util.Base64

@RunWith(Parameterized::class)
class SignatureVerifierTest(private val name: String, private val publicKey: String, private val message: String, private val signature: String, private val isValid: Boolean) {
    @Test
    fun isValidEs256() {
        val decoder = Base64.getDecoder()

        assertEquals(name, isValid, SignatureVerifier().isValidEs256(decoder.decode(publicKey), message.toByteArray(), decoder.decode(signature)))
    }

    companion object {
        @JvmStatic
        @Parameterized.Parameters(name = "{0}")
        fun vectors() = listOf(
            arrayOf<Any>(
                "walletconnect_attestation",
                "BAmy+AzmDm9Z7XfvDphMTvqEtA1gjAtNA57a8piaAfLZKTFwjHtQxGTDR91VsOypcdBfvbo6sAMj5p4Wb+9hRA0=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3MjM0NzE3MjMsImlkIjoiYjNmYmZhMDUxZDJhNjdkNGRmNTYzM2IyMjc0NDAyNTUxMTg1NzQwZGQwMjA3YWM0OWI1M2RiYTcxOTc0YTgzNCIsIm9yaWdpbiI6Imh0dHBzOi8vcmVhY3QtZGFwcC12Mi1naXQtY2hvcmUtdmVyaWZ5LXYyLXNhbXBsZXMtd2FsbGV0Y29ubmVjdDEudmVyY2VsLmFwcCIsImlzU2NhbSI6bnVsbCwiaXNWZXJpZmllZCI6dHJ1ZX0",
                "8RQwiEEfTGn8p3INRdHpi88dpzetKCp3nscfLtWG2cVE2dU0dWgV2ncqnh/RWmygqEnWCPUlH1RMwS1nWbZzrQ==",
                true,
            ),
            arrayOf<Any>(
                "walletconnect_tampered_claims",
                "BAmy+AzmDm9Z7XfvDphMTvqEtA1gjAtNA57a8piaAfLZKTFwjHtQxGTDR91VsOypcdBfvbo6sAMj5p4Wb+9hRA0=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3MjM0NzE3MjQsImlkIjoiYjNmYmZhMDUxZDJhNjdkNGRmNTYzM2IyMjc0NDAyNTUxMTg1NzQwZGQwMjA3YWM0OWI1M2RiYTcxOTc0YTgzNCIsIm9yaWdpbiI6Imh0dHBzOi8vcmVhY3QtZGFwcC12Mi1naXQtY2hvcmUtdmVyaWZ5LXYyLXNhbXBsZXMtd2FsbGV0Y29ubmVjdDEudmVyY2VsLmFwcCIsImlzU2NhbSI6bnVsbCwiaXNWZXJpZmllZCI6dHJ1ZX0",
                "8RQwiEEfTGn8p3INRdHpi88dpzetKCp3nscfLtWG2cVE2dU0dWgV2ncqnh/RWmygqEnWCPUlH1RMwS1nWbZzrQ==",
                false,
            ),
            arrayOf<Any>(
                "wrong_key",
                "BDrasV1mJWvxXNcWA1s/BBRE5RL+0d1k1Lp1WX0g42bxVG0skKg+uroBWVCZ5fP/u9M4THSU3mdZ/dXmXvrpzGc=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "Rxw+dYxJBChbun5TEY7Q9SSt6wdX0lvS+Oew1236cUwxmo8GF+Ib0YP3ccvVug6CC33+8n0IjwYf5VS9OlvlRw==",
                false,
            ),
            arrayOf<Any>(
                "valid_high_s",
                "BB4YUy/UdUwC8wQdnHXOszuD/9gax85P6ILMscmLxYlupGwxHE4v9A3ZajZT5uRURdMt/khuztdcepDGoYiBwKM=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "Rxw+dYxJBChbun5TEY7Q9SSt6wdX0lvS+Oew1236cUzOZXD46B3kL3wIjjQqRfF9sWj7uyoPD37T1HYFwgdACg==",
                true,
            ),
            arrayOf<Any>(
                "valid_r_leading_zero_byte",
                "BB4YUy/UdUwC8wQdnHXOszuD/9gax85P6ILMscmLxYlupGwxHE4v9A3ZajZT5uRURdMt/khuztdcepDGoYiBwKM=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "AK5lhcWODTlRKUF7sUf49Wqk7fRAVbMKW5tpn6C244FQCMjWeTtQeUEYzZt6/orHrp8Z4S4rKKQErKvI72eUsA==",
                true,
            ),
            arrayOf<Any>(
                "valid_r_and_s_high_bit",
                "BB4YUy/UdUwC8wQdnHXOszuD/9gax85P6ILMscmLxYlupGwxHE4v9A3ZajZT5uRURdMt/khuztdcepDGoYiBwKM=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "pF7nUhDUCcinvJ+ks50D1BEif67GgG0cTcXWu4vIAv/Bd+9w/A5fPPdM5oqp0L5cLQFxHdIoKxuP2z5ezzjvIQ==",
                true,
            ),
            arrayOf<Any>(
                "valid_small_r_and_s",
                "BICJ/4tfgSVVvOJRtFA/VJNw038MB+UWRfx1pGlV2JOt/B4f/PLV851ckwf9ER0failrFhImK4x7Evsy86ScyDo=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAUAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQ==",
                true,
            ),
            arrayOf<Any>(
                "small_r_plus_order",
                "BICJ/4tfgSVVvOJRtFA/VJNw038MB+UWRfx1pGlV2JOt/B4f/PLV851ckwf9ER0failrFhImK4x7Evsy86ScyDo=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "/////wAAAAD//////////7zm+q2nF56E87nKwvxjJVYAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQ==",
                false,
            ),
            arrayOf<Any>(
                "small_s_plus_order",
                "BICJ/4tfgSVVvOJRtFA/VJNw038MB+UWRfx1pGlV2JOt/B4f/PLV851ckwf9ER0failrFhImK4x7Evsy86ScyDo=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAX/////AAAAAf//////////vOb6racXnoTzucrC/GMlUg==",
                false,
            ),
            arrayOf<Any>(
                "r_zero",
                "BB4YUy/UdUwC8wQdnHXOszuD/9gax85P6ILMscmLxYlupGwxHE4v9A3ZajZT5uRURdMt/khuztdcepDGoYiBwKM=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAxmo8GF+Ib0YP3ccvVug6CC33+8n0IjwYf5VS9OlvlRw==",
                false,
            ),
            arrayOf<Any>(
                "s_zero",
                "BB4YUy/UdUwC8wQdnHXOszuD/9gax85P6ILMscmLxYlupGwxHE4v9A3ZajZT5uRURdMt/khuztdcepDGoYiBwKM=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "Rxw+dYxJBChbun5TEY7Q9SSt6wdX0lvS+Oew1236cUwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
                false,
            ),
            arrayOf<Any>(
                "r_order",
                "BB4YUy/UdUwC8wQdnHXOszuD/9gax85P6ILMscmLxYlupGwxHE4v9A3ZajZT5uRURdMt/khuztdcepDGoYiBwKM=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "/////wAAAAD//////////7zm+q2nF56E87nKwvxjJVExmo8GF+Ib0YP3ccvVug6CC33+8n0IjwYf5VS9OlvlRw==",
                false,
            ),
            arrayOf<Any>(
                "s_order",
                "BB4YUy/UdUwC8wQdnHXOszuD/9gax85P6ILMscmLxYlupGwxHE4v9A3ZajZT5uRURdMt/khuztdcepDGoYiBwKM=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "Rxw+dYxJBChbun5TEY7Q9SSt6wdX0lvS+Oew1236cUz/////AAAAAP//////////vOb6racXnoTzucrC/GMlUQ==",
                false,
            ),
            arrayOf<Any>(
                "off_curve_point",
                "BB4YUy/UdUwC8wQdnHXOszuD/9gax85P6ILMscmLxYlupGwxHE4v9A3ZajZT5uRURdMt/khuztdcepDGoYiBwKQ=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "Rxw+dYxJBChbun5TEY7Q9SSt6wdX0lvS+Oew1236cUwxmo8GF+Ib0YP3ccvVug6CC33+8n0IjwYf5VS9OlvlRw==",
                false,
            ),
            arrayOf<Any>(
                "zero_point",
                "BAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "Rxw+dYxJBChbun5TEY7Q9SSt6wdX0lvS+Oew1236cUwxmo8GF+Ib0YP3ccvVug6CC33+8n0IjwYf5VS9OlvlRw==",
                false,
            ),
            arrayOf<Any>("identity_encoding", "AA==", "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ", "Rxw+dYxJBChbun5TEY7Q9SSt6wdX0lvS+Oew1236cUwxmo8GF+Ib0YP3ccvVug6CC33+8n0IjwYf5VS9OlvlRw==", false),
            arrayOf<Any>(
                "compressed_key",
                "Ax4YUy/UdUwC8wQdnHXOszuD/9gax85P6ILMscmLxYlu",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "Rxw+dYxJBChbun5TEY7Q9SSt6wdX0lvS+Oew1236cUwxmo8GF+Ib0YP3ccvVug6CC33+8n0IjwYf5VS9OlvlRw==",
                false,
            ),
            arrayOf<Any>(
                "truncated_signature",
                "BB4YUy/UdUwC8wQdnHXOszuD/9gax85P6ILMscmLxYlupGwxHE4v9A3ZajZT5uRURdMt/khuztdcepDGoYiBwKM=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "Rxw+dYxJBChbun5TEY7Q9SSt6wdX0lvS+Oew1236cUwxmo8GF+Ib0YP3ccvVug6CC33+8n0IjwYf5VS9Olvl",
                false,
            ),
            arrayOf<Any>(
                "oversized_signature",
                "BB4YUy/UdUwC8wQdnHXOszuD/9gax85P6ILMscmLxYlupGwxHE4v9A3ZajZT5uRURdMt/khuztdcepDGoYiBwKM=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "Rxw+dYxJBChbun5TEY7Q9SSt6wdX0lvS+Oew1236cUwxmo8GF+Ib0YP3ccvVug6CC33+8n0IjwYf5VS9OlvlRwA=",
                false,
            ),
            arrayOf<Any>(
                "der_signature",
                "BB4YUy/UdUwC8wQdnHXOszuD/9gax85P6ILMscmLxYlupGwxHE4v9A3ZajZT5uRURdMt/khuztdcepDGoYiBwKM=",
                "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6Im1lc3NhZ2UifQ",
                "MEQCIEccPnWMSQQoW7p+UxGO0PUkresHV9Jb0vjnsNdt+nFMAiAxmo8GF+Ib0YP3ccvVug6CC33+8n0IjwYf5VS9OlvlRw==",
                false,
            ),
        )
    }
}
