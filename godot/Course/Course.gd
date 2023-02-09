extends Spatial
class_name Course

export var path: String

func _ready() -> void:
	var spline = Spline.load_from_file(path)

	var vehicle_set = VehicleSet.new()
	add_child(vehicle_set)
	# spawn player
	var vehicle = Vehicle.spawn_basic(spline, VehicleTypes.test_model, spline.get_baked(0.0) + Vector3.UP)
	vehicle.controller = PlayerController.new()
	vehicle_set.add_child(vehicle)
	# add camera for player
	var camera = VehicleCamera.new()
	camera.vehicle = vehicle
	add_child(camera)
	camera.set_initial_pos()
	# for physics testing, add some more vehicles
	var basic = Vehicle.spawn_basic(spline, VehicleTypes.test_model, spline.get_baked(5.0) + Vector3.UP)
	vehicle_set.add_child(basic)
	basic = Vehicle.spawn_basic(spline, VehicleTypes.test_model, spline.get_baked(10.0) + Vector3.UP)
	vehicle_set.add_child(basic)
	basic = Vehicle.spawn_basic(spline, VehicleTypes.test_model, spline.get_baked(15.0) + Vector3.UP)
	vehicle_set.add_child(basic)
	# create line renderer child
	var renderer = CourseRenderer.new()
	renderer.load_course(spline)
	add_child(renderer)
