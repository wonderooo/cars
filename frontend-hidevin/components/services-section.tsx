import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Database, FileX, Shield, Clock } from "lucide-react"

const services = [
    {
        icon: Database,
        title: "Copart Removal",
        description:
            "Complete deletion of your vehicle records from Copart auction database, including all historical data and images.",
    },
    {
        icon: FileX,
        title: "IAAI Removal",
        description:
            "Permanent removal from Insurance Auto Auctions database. All traces of your vehicle history will be eliminated.",
    },
    {
        icon: Shield,
        title: "Privacy Protection",
        description:
            "We ensure your vehicle information is handled with the highest level of security and confidentiality throughout the process.",
    },
    {
        icon: Clock,
        title: "Fast Turnaround",
        description:
            "Most deletions are completed within 5-7 business days. We provide real-time updates on your request status.",
    },
]

export function ServicesSection() {
    return (
        <section id="services" className="py-20 px-4 lg:px-8 bg-muted/30">
            <div className="container mx-auto">
                <div className="max-w-3xl mx-auto text-center mb-16">
                    <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-foreground mb-4 text-balance">
                        Professional Vehicle History Deletion
                    </h2>
                    <p className="text-lg text-muted-foreground text-pretty leading-relaxed">
                        Our comprehensive service removes your vehicle from major auction databases, protecting your privacy and
                        vehicle value.
                    </p>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-5xl mx-auto">
                    {services.map((service, index) => {
                        const Icon = service.icon
                        return (
                            <Card key={index} className="bg-card border-border hover:border-accent/50 transition-colors">
                                <CardHeader>
                                    <div className="h-12 w-12 rounded-lg bg-accent/10 flex items-center justify-center mb-4">
                                        <Icon className="h-6 w-6 text-accent" />
                                    </div>
                                    <CardTitle className="text-xl text-foreground">{service.title}</CardTitle>
                                    <CardDescription className="text-muted-foreground leading-relaxed">
                                        {service.description}
                                    </CardDescription>
                                </CardHeader>
                            </Card>
                        )
                    })}
                </div>
            </div>
        </section>
    )
}
