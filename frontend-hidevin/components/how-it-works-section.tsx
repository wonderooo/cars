import { Card, CardContent } from "@/components/ui/card"

const steps = [
    {
        number: "01",
        title: "Submit Your VIN",
        description: "Enter your 17-digit VIN number and we'll verify if your vehicle appears in Copart or IAAI databases.",
    },
    {
        number: "02",
        title: "We Process Your Request",
        description: "Our team initiates the deletion process with the auction platforms using our established protocols.",
    },
    {
        number: "03",
        title: "Verification & Confirmation",
        description: "We verify complete removal and provide you with confirmation documentation within 5-7 business days.",
    },
]

export function HowItWorksSection() {
    return (
        <section id="how-it-works" className="py-20 px-4 lg:px-8">
            <div className="container mx-auto">
                <div className="max-w-3xl mx-auto text-center mb-16">
                    <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-foreground mb-4 text-balance">
                        Simple Three-Step Process
                    </h2>
                    <p className="text-lg text-muted-foreground text-pretty leading-relaxed">
                        Getting your vehicle history cleared is straightforward and hassle-free.
                    </p>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-3 gap-8 max-w-6xl mx-auto">
                    {steps.map((step, index) => (
                        <div key={index} className="relative">
                            <Card className="bg-card border-border h-full">
                                <CardContent className="pt-6">
                                    <div className="text-6xl font-bold text-accent/20 mb-4">{step.number}</div>
                                    <h3 className="text-xl font-semibold text-foreground mb-3">{step.title}</h3>
                                    <p className="text-muted-foreground leading-relaxed">{step.description}</p>
                                </CardContent>
                            </Card>
                            {index < steps.length - 1 && (
                                <div className="hidden md:block absolute top-1/2 -right-4 w-8 h-0.5 bg-border" />
                            )}
                        </div>
                    ))}
                </div>
            </div>
        </section>
    )
}
