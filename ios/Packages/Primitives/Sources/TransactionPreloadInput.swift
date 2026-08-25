// Copyright (c). Gem Wallet. All rights reserved.

public struct TransactionPreloadInput: Equatable, Hashable, Sendable {
    public let inputType: TransferDataType
    public let senderAddress: String
    public let destinationAddress: String
    public let references: [String]

    public init(inputType: TransferDataType, senderAddress: String, destinationAddress: String, references: [String] = []) {
        self.inputType = inputType
        self.senderAddress = senderAddress
        self.destinationAddress = destinationAddress
        self.references = references
    }
}
