class_name npc_base
extends KinematicBody

var camRotation:Vector2 = Vector2.ZERO;

func _ready():
	Persistant.get_node("controllerNPCS").AddNPC(self.get_instance_id());

func _process(delta):
	Network();

func Network():
	var network = Persistant.get_node("controllerNetwork");
	if (network.HasTicked()):
		network.rpc("NPCBaseInfo", self.get_instance_id(), global_transform.origin, camRotation);

func _exit_tree():
	Persistant.get_node("controllerNPCS").RemoveNPCWithID(self.get_instance_id());
