import Components
import struct Gemstone.GemWalletRow
import Style
import SwiftUI

public struct WalletBarView: View {
    private let row: GemWalletRow
    private let action: (() -> Void)?

    public init(
        row: GemWalletRow,
        action: (() -> Void)? = nil,
    ) {
        self.row = row
        self.action = action
    }

    public var body: some View {
        Button {
            action?()
        } label: {
            HStack(spacing: .small) {
                AssetImageView(assetImage: row.avatarImage, size: .large)

                Text(row.name)
                    .foregroundStyle(Colors.black)
                    .fontWeight(.medium)
                    .font(.body)
                    .lineLimit(1)

                Images.System.chevronDown
                    .resizable()
                    .frame(width: 11, height: 6)
                    .fontWeight(.medium)
                    .foregroundStyle(Colors.gray)
            }
            .padding(.small)
        }
        .buttonStyle(.plain)
        .accessibilityIdentifier("walletBar")
    }
}
