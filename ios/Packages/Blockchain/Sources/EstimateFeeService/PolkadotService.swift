// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives
import SwiftHTTPClient
import WalletCore

internal import GemstonePrimitives

final class PolkadotService: Sendable {
    init() {}

    func feePayload(input: TransactionInput) throws -> String {
        guard case let .polkadot(
            sequence,
            genesisHash,
            blockHash,
            blockNumber,
            specVersion,
            transactionVersion,
            period,
        ) = input.metadata else {
            throw AnyError("")
        }

        let input = try PolkadotSigningInput.with {
            $0.genesisHash = try genesisHash.encodedData()
            $0.blockHash = try blockHash.encodedData()
            $0.nonce = sequence
            $0.specVersion = UInt32(specVersion)
            $0.network = CoinType.polkadot.ss58Prefix
            $0.transactionVersion = UInt32(transactionVersion)
            $0.privateKey = PrivateKey().data
            $0.era = PolkadotEra.with {
                $0.blockNumber = blockNumber
                $0.period = period
            }
            $0.chargeNativeAsAssetTxPayment = true
            $0.balanceCall.transfer = PolkadotBalance.Transfer.with {
                $0.toAddress = input.destinationAddress
                $0.value = input.value.magnitude.serialize()
                $0.callIndices = .with {
                    $0.variant = .custom(.with {
                        $0.moduleIndex = 0x0A
                        $0.methodIndex = 0x00
                    })
                }
            }
        }
        let output: PolkadotSigningOutput = AnySigner.sign(input: input, coin: .polkadot)
        return output.encoded.hexString.append0x
    }
}

extension PolkadotService: GemGatewayEstimateFee {
    func getFee(chain _: Gemstone.Chain, input _: Gemstone.GemTransactionLoadInput) async throws -> Gemstone.GemTransactionLoadFee? {
        .none
    }

    func getFeeData(chain _: Gemstone.Chain, input: GemTransactionLoadInput) async throws -> String? {
        try feePayload(input: input.map())
    }
}
