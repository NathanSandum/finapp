let armed = false;

export const overviewEntrance = {
	arm() {
		armed = true;
	},
	consume(): boolean {
		if (!armed) return false;
		armed = false;
		return true;
	}
};
