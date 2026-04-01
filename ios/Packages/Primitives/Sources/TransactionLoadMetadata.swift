// Copyright (c). Gem Wallet. All rights reserved.

public struct HyperliquidOrder: Sendable {
    public let approveAgentRequired: Bool
    public let approveReferralRequired: Bool
    public let approveBuilderRequired: Bool
    public let builderFeeBps: UInt32
    public let agentAddress: String
    public let agentPrivateKey: String

    public init(
        approveAgentRequired: Bool,
        approveReferralRequired: Bool,
        approveBuilderRequired: Bool,
        builderFeeBps: UInt32,
        agentAddress: String,
        agentPrivateKey: String,
    ) {
        self.approveAgentRequired = approveAgentRequired
        self.approveReferralRequired = approveReferralRequired
        self.approveBuilderRequired = approveBuilderRequired
        self.builderFeeBps = builderFeeBps
        self.agentAddress = agentAddress
        self.agentPrivateKey = agentPrivateKey
    }
}

public enum TransactionLoadMetadata: Sendable {
    case none
    case solana(
        senderTokenAddress: String?,
        recipientTokenAddress: String?,
        tokenProgram: SolanaTokenProgramId?,
        blockHash: String,
    )
    case ton(
        senderTokenAddress: String?,
        recipientTokenAddress: String?,
        sequence: UInt64,
    )
    case cosmos(
        accountNumber: UInt64,
        sequence: UInt64,
        chainId: String,
    )
    case bitcoin(utxos: [UTXO])
    case zcash(utxos: [UTXO], branchId: String)
    case cardano(utxos: [UTXO])
    case evm(nonce: UInt64, chainId: UInt64, contractCall: ContractCallData? = nil)
    case near(
        sequence: UInt64,
        blockHash: String,
    )
    case stellar(sequence: UInt64, isDestinationAddressExist: Bool)
    case xrp(sequence: UInt64, blockNumber: UInt64)
    case algorand(
        sequence: UInt64,
        blockHash: String,
        chainId: String,
    )
    case aptos(sequence: UInt64, data: String? = nil)
    case polkadot(
        sequence: UInt64,
        genesisHash: String,
        blockHash: String,
        blockNumber: UInt64,
        specVersion: UInt64,
        transactionVersion: UInt64,
        period: UInt64,
    )
    case tron(
        blockNumber: UInt64,
        blockVersion: UInt64,
        blockTimestamp: UInt64,
        transactionTreeRoot: String,
        parentHash: String,
        witnessAddress: String,
        stakeData: TronStakeData,
    )
    case sui(messageBytes: String)
    case hyperliquid(order: HyperliquidOrder?)
}

public extension TransactionLoadMetadata {
    func getSequence() throws -> UInt64 {
        switch self {
        case let .ton(_, _, sequence),
             let .cosmos(_, sequence, _),
             let .near(sequence, _),
             let .stellar(sequence, _),
             let .xrp(sequence, _),
             let .algorand(sequence, _, _),
             let .aptos(sequence, _),
             let .polkadot(sequence, _, _, _, _, _, _),
             let .evm(sequence, _, _):
            return sequence
        case .none, .bitcoin, .zcash, .cardano, .tron, .solana, .sui, .hyperliquid:
            throw AnyError("Sequence not available for this metadata type")
        }
    }

    func getBlockNumber() throws -> UInt64 {
        switch self {
        case let .polkadot(_, _, _, blockNumber, _, _, _),
             let .tron(blockNumber, _, _, _, _, _, _),
             let .xrp(_, blockNumber):
            return blockNumber
        default:
            throw AnyError("Block number not available for this metadata type")
        }
    }

    func getBlockHash() throws -> String {
        switch self {
        case let .solana(_, _, _, blockHash),
             let .near(_, blockHash),
             let .algorand(_, blockHash, _),
             let .polkadot(_, _, blockHash, _, _, _, _):
            return blockHash
        default:
            throw AnyError("Block hash not available for this metadata type")
        }
    }

    func getChainId() throws -> String {
        switch self {
        case let .cosmos(_, _, chainId):
            return chainId
        case let .algorand(_, _, chainId):
            return chainId
        case let .evm(_, chainId, _):
            return String(chainId)
        default:
            throw AnyError("Chain ID not available for this metadata type")
        }
    }

    func getUtxos() throws -> [UTXO] {
        switch self {
        case let .bitcoin(utxos),
             let .zcash(utxos, _),
             let .cardano(utxos):
            return utxos
        default:
            throw AnyError("UTXOs not available for this metadata type")
        }
    }

    func getIsDestinationAddressExist() throws -> Bool {
        switch self {
        case let .stellar(_, isDestinationAddressExist):
            return isDestinationAddressExist
        default:
            throw AnyError("Destination existence flag not available for this metadata type")
        }
    }

    func getAccountNumber() throws -> UInt64 {
        switch self {
        case let .cosmos(accountNumber, _, _):
            return accountNumber
        default:
            throw AnyError("Account number not available for this metadata type")
        }
    }

    func getMessageBytes() throws -> String {
        switch self {
        case let .sui(messageBytes):
            return messageBytes
        default:
            throw AnyError("Message bytes not available for this metadata type")
        }
    }

    func senderTokenAddress() throws -> String? {
        switch self {
        case let .ton(senderTokenAddress, _, _):
            return senderTokenAddress
        default:
            throw AnyError("Sender token address not available for this metadata type")
        }
    }

    func getData() throws -> String {
        let data: String? = switch self {
        case let .aptos(_, data): data
        default: .none
        }
        guard let data else {
            throw AnyError("Data not available for this metadata type")
        }
        return data
    }
}
