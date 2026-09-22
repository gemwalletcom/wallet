import GemstonePrimitives
import InfoSheet
import Style
import SwiftUI

struct JailbreakWarningView: View {
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        InfoSheetScene(
            model: InfoSheetModel(
                title: "Security Warning",
                description: "Your device appears to be jailbroken. This may put your wallet and funds at risk.",
                image: .image(Image(systemName: SystemImage.exclamationmarkTriangle)),
                button: .url(AppUrl.docs(.rootedDevice), title: "Learn More"),
                secondaryButtons: [
                    .action(title: "Continue Anyway", action: { dismiss() }),
                ],
            ),
        )
    }
}
