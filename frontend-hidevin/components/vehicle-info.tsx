import {Card, CardContent, CardHeader, CardTitle} from "@/components/ui/card"
import {Badge} from "@/components/ui/badge"
import {Separator} from "@/components/ui/separator"
import {Tabs, TabsContent, TabsList, TabsTrigger} from "@/components/ui/tabs"
import {AlertTriangle, Calendar, Cog, DollarSign, Fuel, Gauge, Key, MapPin, Palette, Wrench} from "lucide-react"
import {Vehicle} from "@/app/vin/[vin]/page";

interface VehicleInfoProps {
    vehicle: Vehicle
}

export function VehicleInfo({vehicle}: VehicleInfoProps) {
    const formatCurrency = (amount: number) => {
        return new Intl.NumberFormat("en-US", {
            style: "currency",
            currency: vehicle.currency,
        }).format(amount)
    }

    const formatDate = (date: Date | null) => {
        if (!date) return "N/A"
        return new Intl.DateTimeFormat("en-US", {
            year: "numeric",
            month: "long",
            day: "numeric",
        }).format(date)
    }

    return (
        <Tabs defaultValue="overview" className="w-full">
            <TabsList className="grid w-full grid-cols-3">
                <TabsTrigger value="overview">Overview</TabsTrigger>
                <TabsTrigger value="specifications">Specifications</TabsTrigger>
                <TabsTrigger value="damage">Damage Report</TabsTrigger>
            </TabsList>

            <TabsContent value="overview" className="space-y-4">
                <Card>
                    <CardHeader>
                        <CardTitle>Vehicle Overview</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="grid sm:grid-cols-2 gap-4">
                            <div className="flex items-start gap-3">
                                <Gauge className="h-5 w-5 text-muted-foreground mt-0.5"/>
                                <div>
                                    <p className="text-sm text-muted-foreground">Odometer</p>
                                    <p className="font-semibold">{vehicle.odometer.toLocaleString()} miles</p>
                                    {vehicle.odometerStatus && (
                                        <Badge variant="secondary" className="mt-1">
                                            {vehicle.odometerStatus}
                                        </Badge>
                                    )}
                                </div>
                            </div>

                            <div className="flex items-start gap-3">
                                <MapPin className="h-5 w-5 text-muted-foreground mt-0.5"/>
                                <div>
                                    <p className="text-sm text-muted-foreground">Location</p>
                                    <p className="font-semibold">
                                        {vehicle.state}, {vehicle.country}
                                    </p>
                                </div>
                            </div>

                            <div className="flex items-start gap-3">
                                <Calendar className="h-5 w-5 text-muted-foreground mt-0.5"/>
                                <div>
                                    <p className="text-sm text-muted-foreground">Sale Date</p>
                                    <p className="font-semibold">{vehicle.saleDate}</p>
                                </div>
                            </div>

                            <div className="flex items-start gap-3">
                                <DollarSign className="h-5 w-5 text-muted-foreground mt-0.5"/>
                                <div>
                                    <p className="text-sm text-muted-foreground">Sold For</p>
                                    <p className="font-semibold">{"N/A"}</p>
                                </div>
                            </div>
                        </div>

                        <Separator/>

                        <div className="space-y-3">
                            <div className="flex justify-between">
                                <span className="text-muted-foreground">Estimated Retail Value</span>
                                <span className="font-semibold">{formatCurrency(vehicle.estimatedRetailValue)}</span>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-muted-foreground">Estimated Repair Cost</span>
                                <span
                                    className="font-semibold text-destructive">{formatCurrency(vehicle.estimatedRepairCost)}</span>
                            </div>
                        </div>
                    </CardContent>
                </Card>
            </TabsContent>

            <TabsContent value="specifications" className="space-y-4">
                <Card>
                    <CardHeader>
                        <CardTitle>Technical Specifications</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="grid sm:grid-cols-2 gap-4">
                            <div className="flex items-start gap-3">
                                <Cog className="h-5 w-5 text-muted-foreground mt-0.5"/>
                                <div>
                                    <p className="text-sm text-muted-foreground">Engine</p>
                                    <p className="font-semibold">{vehicle.engineName || "N/A"}</p>
                                    {vehicle.engineCylinders && (
                                        <p className="text-sm text-muted-foreground">{vehicle.engineCylinders} Cylinders</p>
                                    )}
                                </div>
                            </div>

                            <div className="flex items-start gap-3">
                                <Wrench className="h-5 w-5 text-muted-foreground mt-0.5"/>
                                <div>
                                    <p className="text-sm text-muted-foreground">Transmission</p>
                                    <p className="font-semibold">{vehicle.transmission || "N/A"}</p>
                                </div>
                            </div>

                            <div className="flex items-start gap-3">
                                <Fuel className="h-5 w-5 text-muted-foreground mt-0.5"/>
                                <div>
                                    <p className="text-sm text-muted-foreground">Fuel Type</p>
                                    <p className="font-semibold">{vehicle.fuelType || "N/A"}</p>
                                </div>
                            </div>

                            <div className="flex items-start gap-3">
                                <Cog className="h-5 w-5 text-muted-foreground mt-0.5"/>
                                <div>
                                    <p className="text-sm text-muted-foreground">Drive Type</p>
                                    <p className="font-semibold">{vehicle.driveType || "N/A"}</p>
                                </div>
                            </div>

                            <div className="flex items-start gap-3">
                                <Palette className="h-5 w-5 text-muted-foreground mt-0.5"/>
                                <div>
                                    <p className="text-sm text-muted-foreground">Color</p>
                                    <p className="font-semibold">{vehicle.color}</p>
                                </div>
                            </div>

                            <div className="flex items-start gap-3">
                                <Key className="h-5 w-5 text-muted-foreground mt-0.5"/>
                                <div>
                                    <p className="text-sm text-muted-foreground">Keys Status</p>
                                    <p className="font-semibold">{vehicle.keysStatus || "N/A"}</p>
                                </div>
                            </div>
                        </div>

                        <Separator/>

                        <div className="space-y-2">
                            <div className="flex justify-between">
                                <span className="text-muted-foreground">Vehicle Type</span>
                                <Badge variant="outline">{vehicle.vehicleType}</Badge>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-muted-foreground">VIN</span>
                                <span className="font-mono text-sm">{vehicle.vin}</span>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-muted-foreground">Lot Number</span>
                                <span className="font-mono text-sm">{vehicle.lotNumber}</span>
                            </div>
                        </div>
                    </CardContent>
                </Card>
            </TabsContent>

            <TabsContent value="damage" className="space-y-4">
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <AlertTriangle className="h-5 w-5 text-destructive"/>
                            Damage Report
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="space-y-4">
                            <div>
                                <p className="text-sm text-muted-foreground mb-2">Primary Damage</p>
                                <Badge variant="destructive" className="text-base px-3 py-1">
                                    {vehicle.mainDamage}
                                </Badge>
                            </div>

                            {vehicle.otherDamage && (
                                <div>
                                    <p className="text-sm text-muted-foreground mb-2">Secondary Damage</p>
                                    <Badge variant="outline" className="text-base px-3 py-1">
                                        {vehicle.otherDamage}
                                    </Badge>
                                </div>
                            )}
                        </div>

                        <Separator/>

                        <div className="bg-muted/50 p-4 rounded-lg space-y-2">
                            <p className="font-semibold">Damage Assessment</p>
                            <p className="text-sm text-muted-foreground">
                                This vehicle has sustained damage and may require significant repairs. The estimated
                                repair cost is{" "}
                                <span
                                    className="font-semibold text-foreground">{formatCurrency(vehicle.estimatedRepairCost)}</span>.
                            </p>
                        </div>
                    </CardContent>
                </Card>
            </TabsContent>
        </Tabs>
    )
}
