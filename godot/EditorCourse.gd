extends Spatial


# Declare member variables here. Examples:
# var a = 2
# var b = "text"


var spline := Spline.load_from_file('res://Course/Data/Test1.bin')

# Called when the node enters the scene tree for the first time.
func _ready():
	var renderer = CourseRenderer.new()
	renderer.load_course(spline)
	add_child(renderer)
	var camera = EditorCamera.new()
	add_child(camera)


# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass
